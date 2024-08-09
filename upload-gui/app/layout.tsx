import type { Metadata } from 'next';
import './globals.css';
import PageTheme from './page-theme';
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
        <PageTheme>
          {children}
        </PageTheme>
      </body>
    </html>
  )
}