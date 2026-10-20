import "./ai.css";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";
import "./integrations.css";
import "./records.css";
import "./controls.css";
import "./workbench.css";
import "./extensions.css";
import { invoke } from "@tauri-apps/api/core";
import { request, errorText, type AppTheme } from "./api";
import { applyTheme } from "./theme";
const root = ReactDOM.createRoot(document.getElementById("root")!);
async function bootstrap() {
  try {
    const initialTheme = await request<AppTheme>("get_theme");
    applyTheme(initialTheme);
    root.render(
      <React.StrictMode>
        <App initialTheme={initialTheme} />
      </React.StrictMode>,
    );
  } catch (error) {
    // Do not display the application with guessed saved settings.
    root.render(
      <main className="startup-error">
        <h1>Workspace could not open</h1>
        <p role="alert">{errorText(error)}</p>
        <button className="button primary" onClick={() => void bootstrap()}>
          Retry connection
        </button>
      </main>,
    );
    await invoke("apply_window_theme", { theme: "reconx" }).catch(() => {});
  }
}
void bootstrap();
