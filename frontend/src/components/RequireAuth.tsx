import { Navigate, Outlet } from "react-router-dom";
import { useAuth } from "../auth";

export function RequireAuth() {
  const { session } = useAuth();
  if (!session) return <Navigate to="/login" replace />;
  return <Outlet />;
}
