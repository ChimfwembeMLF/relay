use axum::{
    http::header,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Router,
};

const OPENAPI_YAML: &str = include_str!("../../openapi/openapi.yaml");

const SWAGGER_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Payment Relay API</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.17.14/swagger-ui.css">
  <style>
    html { box-sizing: border-box; overflow-y: scroll; }
    body { margin: 0; background: #fafafa; }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5.17.14/swagger-ui-bundle.js" crossorigin></script>
  <script>
    window.onload = function () {
      window.ui = SwaggerUIBundle({
        url: "/api-docs/openapi.yaml",
        dom_id: "#swagger-ui",
        deepLinking: true,
        presets: [SwaggerUIBundle.presets.apis],
        layout: "BaseLayout",
      });
    };
  </script>
</body>
</html>"##;

pub fn mount<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .route("/swagger-ui", get(swagger_ui))
        .route("/swagger-ui/", get(swagger_ui))
        .route("/api-docs/openapi.yaml", get(openapi_yaml))
        .route("/docs", get(redirect_to_swagger))
}

async fn swagger_ui() -> Html<&'static str> {
    Html(SWAGGER_HTML)
}

async fn openapi_yaml() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/yaml")], OPENAPI_YAML)
}

async fn redirect_to_swagger() -> Redirect {
    Redirect::temporary("/swagger-ui/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_yaml_is_non_empty() {
        assert!(OPENAPI_YAML.contains("openapi:"));
        assert!(OPENAPI_YAML.contains("/payments"));
        assert!(OPENAPI_YAML.contains("/api/pay/{reference}"));
    }
}
