import { CustomContentProps, SnackbarProvider } from 'notistack';

import { notifications } from '@/config';

import Alert from '@mui/material/Alert';
import AlertTitle from '@mui/material/AlertTitle';
import { Ref, forwardRef } from 'react';
import Notifier from './Notifier';

const ErrorNotification = forwardRef(function ErrorNotification(
  { message }: CustomContentProps,
  ref: Ref<HTMLDivElement>,
) {
  return (
    <Alert ref={ref} severity="error">
      <AlertTitle>Klaida</AlertTitle>
      {message}
    </Alert>
  );
});

const InfoNotification = forwardRef(function InfoNotification(
  { message }: CustomContentProps,
  ref: Ref<HTMLDivElement>,
) {
  return (
    <Alert ref={ref} severity="info">
      <AlertTitle>Info</AlertTitle>
      {message}
    </Alert>
  );
});

function Notifications() {
  return (
    <SnackbarProvider
      maxSnack={notifications.maxSnack}
      Components={{
        infoNotification: InfoNotification,
        errorNotification: ErrorNotification,
      }}
    >
      <Notifier />
    </SnackbarProvider>
  );
}

export default Notifications;
