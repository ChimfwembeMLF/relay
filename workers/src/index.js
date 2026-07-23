import { Queue, Worker } from "bullmq";
import Redis from "ioredis";

const REDIS_URL = process.env.REDIS_URL;
if (!REDIS_URL) {
  console.error("REDIS_URL is required");
  process.exit(1);
}

const INBOX_KEY = "relay:inbox:webhooks";
const QUEUE_NAME = "webhooks";

const connection = new Redis(REDIS_URL, { maxRetriesPerRequest: null });
const queue = new Queue(QUEUE_NAME, { connection });

const worker = new Worker(
  QUEUE_NAME,
  async (job) => {
    const { url, body, signature, event_type, transaction_id } = job.data;
    const res = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Relay-Signature": signature,
      },
      body,
    });

    if (!res.ok) {
      const text = await res.text().catch(() => "");
      throw new Error(
        `webhook ${event_type} for ${transaction_id} failed: HTTP ${res.status} ${text}`,
      );
    }

    console.log(
      JSON.stringify({
        msg: "webhook delivered",
        event_type,
        transaction_id,
        attempt: job.attemptsMade + 1,
      }),
    );
  },
  {
    connection,
    concurrency: 5,
  },
);

worker.on("failed", (job, err) => {
  console.error(
    JSON.stringify({
      msg: "webhook job failed",
      jobId: job?.id,
      error: err.message,
      attempts: job?.attemptsMade,
    }),
  );
});

/** Move Rust LPUSH inbox jobs into BullMQ for retries / observability. */
async function drainInbox() {
  for (;;) {
    try {
      const raw = await connection.brpop(INBOX_KEY, 5);
      if (!raw) continue;
      const payload = JSON.parse(raw[1]);
      await queue.add("deliver", payload, {
        jobId: payload.job_id,
        attempts: 5,
        backoff: { type: "exponential", delay: 2000 },
        removeOnComplete: 1000,
        removeOnFail: 5000,
      });
      console.log(
        JSON.stringify({
          msg: "enqueued to bullmq",
          queue: QUEUE_NAME,
          job_id: payload.job_id,
          event_type: payload.event_type,
        }),
      );
    } catch (err) {
      console.error("inbox drain error", err);
      await new Promise((r) => setTimeout(r, 1000));
    }
  }
}

console.log(
  JSON.stringify({
    msg: "relay webhook worker started",
    redis: REDIS_URL.replace(/:[^:@/]+@/, ":****@"),
    inbox: INBOX_KEY,
    queue: QUEUE_NAME,
  }),
);

drainInbox().catch((err) => {
  console.error(err);
  process.exit(1);
});
