import type { OptionsObject, SnackbarKey, SnackbarMessage } from 'notistack';

interface Notification {
  message: SnackbarMessage;
  options: OptionsObject;
  dismissed: boolean;
}

declare module 'notistack' {
  export interface VariantOverrides {
    errorNotification: {
      message?: string;
    };
    infoNotification: {
      message?: string;
    };
  }
}

type Actions = {
  push: (notification: Partial<Notification>) => SnackbarKey;
  close: (key: SnackbarKey, dismissAll?: boolean) => void;
  remove: (key: SnackbarKey) => void;
};

export type { Actions, Notification };

