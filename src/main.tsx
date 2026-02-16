// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

// Disable default context menu globally for a more native feel
document.addEventListener('contextmenu', (e) => e.preventDefault());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
