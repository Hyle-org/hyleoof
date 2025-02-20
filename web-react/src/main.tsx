import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import App from "./App.tsx";
import { MetaMaskProvider } from "./hooks/MetamaskContext.tsx";

import { dark } from './config/theme';
import { ThemeProvider } from "styled-components";
import { NotificationProvider } from "./hooks/NotificationContext.tsx";
import NotificationList from "./components/ui/NotificationList.tsx";
createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider theme={dark}>
      <NotificationProvider>
        <MetaMaskProvider>
          <App />
          <NotificationList />
        </MetaMaskProvider>
      </NotificationProvider>
    </ThemeProvider>
  </StrictMode>,
);
