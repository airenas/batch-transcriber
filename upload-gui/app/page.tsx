"use client"

import { useTheme } from 'next-themes';
import { useEffect } from 'react';
import 'react-toastify/dist/ReactToastify.css';
import Header from './components/Header';
import Upload from './components/Upload';
import { isDark } from './utils';

export default function Page() {
  const { theme } = useTheme()

  useEffect(() => {

  }, []);

  return (
    <div className={`min-h-screen ${isDark(theme) ? 'bg-gray-900 text-white' : 'bg-white text-black'}`}>
      <Header showHomeButton={false} />
      <main className="p-8">
        <Upload />
      </main>
    </div>
  );
}