"use client"

import { NextUIProvider } from '@nextui-org/react';
import { ThemeProvider as NextThemesProvider } from 'next-themes';
import 'react-toastify/dist/ReactToastify.css';

export default function PageTheme({ children }: { children: React.ReactNode }) {
  function getTheme(): string {
    return 'light';
    // return localStorage.getItem('theme') || 'light';
  }

  return (
    <div>
      <NextThemesProvider
        attribute="class"
        defaultTheme={getTheme()}
      >
        <NextUIProvider>
          {children}
        </NextUIProvider>
      </NextThemesProvider>
    </div>
  );
}

