import { useState } from "react";
import { api } from "./api";

export default function AuthForm({ onLoggedIn }) {
  const [mode, setMode] = useState("login"); // "login" | "register"
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [message, setMessage] = useState(null); // { text, tone: "error" | "info" }

  async function handleSubmit(e) {
    e.preventDefault();
    setMessage(null);

    try {
      if (mode === "register") {
        await api.register(username, password);
        setMode("login");
        setMessage({ text: "Account created — now log in.", tone: "info" });
        return;
      }

      await api.login(username, password);
      onLoggedIn();
    } catch (err) {
      setMessage({ text: err.message, tone: "error" });
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      <h1 className="ledger-title">{mode === "login" ? "Sign in" : "New account"}</h1>
      <p className="ledger-subtitle">
        {mode === "login" ? "Pick up where you left off." : "A few details to get started."}
      </p>
      <hr className="ledger-rule" />

      <label className="field">
        <span className="field-label">username</span>
        <input
          className="field-input"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          autoComplete="username"
        />
      </label>

      <label className="field">
        <span className="field-label">password</span>
        <input
          className="field-input"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoComplete={mode === "login" ? "current-password" : "new-password"}
        />
      </label>

      {message && (
        <p className={`form-message ${message.tone === "info" ? "info" : ""}`}>{message.text}</p>
      )}

      <button type="submit" className="btn auth-submit">
        {mode === "login" ? "Log in" : "Create account"}
      </button>

      <p className="auth-switch">
        {mode === "login" ? "No account yet? " : "Already have one? "}
        <button
          type="button"
          className="link-btn"
          onClick={() => {
            setMode(mode === "login" ? "register" : "login");
            setMessage(null);
          }}
        >
          {mode === "login" ? "Register" : "Log in"}
        </button>
      </p>
    </form>
  );
}