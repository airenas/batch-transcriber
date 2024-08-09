"use client"

import { useTheme } from 'next-themes';
import { ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import Header from './components/Header';
import Upload from './components/Upload';
import { isDark } from './utils';

export default function Page() {
  const { theme } = useTheme()

  const serverUrl = process.env.NEXT_PUBLIC_SERVER_URL || 'http://localhost:8001/upload';

  return (
    <div>
      <div className={`min-h-screen ${isDark(theme) ? 'bg-gray-900 text-white' : 'bg-white text-black'}`}>
        <Header showHomeButton={false} />
        <main className="p-8">
          <Upload serverUrl={serverUrl} />
        </main>
      </div>
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
    </div>
  );
}