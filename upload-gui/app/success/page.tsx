"use client"

import { useTheme } from 'next-themes';
import { Suspense } from 'react';
import Header from '../components/Header';
import Success from '../components/Success';
import { isDark } from '../utils';


export default function Page() {
  const { theme } = useTheme()

  return (
    <div className={`min-h-screen ${isDark(theme) ? 'bg-gray-900 text-white' : 'bg-white text-black'}`}>
      <Header showHomeButton={true} />
      <main className="p-8">
        <Suspense>
          <Success />
        </Suspense>
      </main>
    </div>
  );
}
