import { NextUIProvider } from '@nextui-org/react';
import type { Metadata } from 'next';
import { ThemeProvider as NextThemesProvider } from 'next-themes';
import { ToastContainer } from 'react-toastify';
import './globals.css';
import VersionLogger from './version-logger';

export const metadata: Metadata = {
  title: 'DiPolis Audio saugykla',
  description: 'DiPolis audio failų saugykla',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <VersionLogger />
      <body>
        <NextThemesProvider
          defaultTheme="system"
          attribute="class"
        >
          <NextUIProvider>
            {children}
          </NextUIProvider>
        </NextThemesProvider>

        <ToastContainer
          position="bottom-center"
          autoClose={5000}
          hideProgressBar
          newestOnTop
          closeOnClick
          rtl={false}
          pauseOnFocusLoss
          draggable
          pauseOnHover
        />

      </body>

    </html>
  )
}