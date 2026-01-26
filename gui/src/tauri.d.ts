declare global {
  interface Window {
    __TAURI__?: {
      window: {
        getCurrent: () => {
          close: () => Promise<void>;
          minimize: () => Promise<void>;
          destroy: () => void;
        };
      };
    };
  }
}

export {};
