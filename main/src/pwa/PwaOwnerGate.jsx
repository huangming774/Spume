import { useCallback, useEffect, useMemo, useState } from "react";
import { authMe } from "../services/authApi";
import {
  getAppAccessState,
  shouldRequireOwnerLogin,
  shouldShowOwnerLoginActions,
  userHasOwnerAppAccess,
} from "./appAccessGate";

const openOwnerLogin = () => {
  window.dispatchEvent(
    new CustomEvent("blog-ai-open", {
      detail: { openAuth: true, mode: "login" },
    }),
  );
};

const PwaOwnerGate = ({ children }) => {
  const hostname = typeof window === "undefined" ? "" : window.location.hostname;
  const requiresOwnerLogin = shouldRequireOwnerLogin({ hostname });
  const [auth, setAuth] = useState(null);
  const [isLoading, setIsLoading] = useState(requiresOwnerLogin);
  const [error, setError] = useState("");

  const refreshAuth = useCallback(async () => {
    if (!requiresOwnerLogin) return;

    setIsLoading(true);
    setError("");
    try {
      const result = await authMe();
      setAuth(result.ok ? result.data : { loggedIn: false });
      if (!result.ok) setError("Could not confirm the owner session. Try again.");
    } catch {
      setAuth((current) =>
        userHasOwnerAppAccess(current) ? current : { loggedIn: false },
      );
      setError("The API is not reachable right now. Check /api and retry.");
    } finally {
      setIsLoading(false);
    }
  }, [requiresOwnerLogin]);

  useEffect(() => {
    refreshAuth();
  }, [refreshAuth]);

  useEffect(() => {
    if (!requiresOwnerLogin) return undefined;

    const onAuthChanged = () => refreshAuth();
    const onFocus = () => refreshAuth();
    window.addEventListener("blog-auth-state-changed", onAuthChanged);
    window.addEventListener("focus", onFocus);
    return () => {
      window.removeEventListener("blog-auth-state-changed", onAuthChanged);
      window.removeEventListener("focus", onFocus);
    };
  }, [refreshAuth, requiresOwnerLogin]);

  const accessState = useMemo(
    () => getAppAccessState({ hostname, auth, isLoading }),
    [auth, hostname, isLoading],
  );

  if (accessState === "allowed") return children;

  const showLoginActions = shouldShowOwnerLoginActions(accessState);

  return (
    <main className="app-owner-gate" aria-busy={accessState === "loading"}>
      <section className="app-owner-gate-panel" aria-labelledby="app-owner-gate-title">
        <div className="app-owner-gate-mark">T</div>
        <p className="app-owner-gate-kicker">Personal PWA</p>
        <h1 id="app-owner-gate-title">
          {accessState === "loading" ? "Checking owner session" : "Owner login required"}
        </h1>
        <p className="app-owner-gate-copy">
          {accessState === "loading"
            ? "The app is confirming your existing owner session before opening the console."
            : "This installed app opens only after the owner account is logged in and the security check is complete."}
        </p>
        {auth?.loggedIn && !auth?.user?.isOwner ? (
          <p className="app-owner-gate-warning">
            The current account is not the owner account. Sign out and switch accounts.
          </p>
        ) : null}
        {auth?.user?.isOwner && !auth?.unlimited ? (
          <p className="app-owner-gate-warning">
            The owner account still needs the security check in the AI login panel.
          </p>
        ) : null}
        {error ? <p className="app-owner-gate-warning">{error}</p> : null}
        {showLoginActions ? (
          <div className="app-owner-gate-actions">
            <button type="button" onClick={openOwnerLogin} className="app-owner-gate-primary">
              Open AI login
            </button>
            <button type="button" onClick={refreshAuth} className="app-owner-gate-secondary">
              Recheck session
            </button>
          </div>
        ) : null}
      </section>
    </main>
  );
};

export default PwaOwnerGate;
