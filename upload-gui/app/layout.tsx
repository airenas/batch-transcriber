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

  const basePath = process.env.NEXT_PUBLIC_BASE_PATH || '__BASE_PATH__'
  console.log(`basePath: ${basePath}`)

  return (
    <html lang="lt">
      <head>
        <base href={basePath} />
      </head>
      <body>
        <VersionLogger />
        <PageTheme>
          {children}
        </PageTheme>
      </body>
    </html>
  )
}