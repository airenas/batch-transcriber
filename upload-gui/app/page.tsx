"use client"

import { useEffect, useState } from 'react';
import Header from './components/Header';
import Upload from './components/Upload';
import { ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

export default function Page() {
  const [isNightMode, setIsNightMode] = useState(false);

  // Optionally, load the initial night mode state from localStorage
  useEffect(() => {
    const savedMode = localStorage.getItem('night-mode') === 'true';
    setIsNightMode(savedMode);
  }, []);

  // Toggle night mode and save to localStorage
  const toggleNightMode = () => {
    const newMode = !isNightMode;
    setIsNightMode(newMode);
    localStorage.setItem('night-mode', newMode ? "true" : "false");
  };

  return (
    <div className={`min-h-screen ${isNightMode ? 'bg-gray-900 text-white' : 'bg-white text-black'}`}>
      <Header isNightMode={isNightMode} toggleNightMode={toggleNightMode} showHomeButton={false} />
      <main className="p-8">
        <Upload isNightMode={isNightMode} />
      </main>
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