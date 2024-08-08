"use client"

import { useEffect, useState } from 'react';
import Header from '../components/Header';
import Success from '../components/Success';
import { useSearchParams } from 'next/navigation'


export default function Page() {
  const [isNightMode, setIsNightMode] = useState(false);

  const searchParams = useSearchParams();
  const id = searchParams.get('id')


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
      <Header isNightMode={isNightMode} toggleNightMode={toggleNightMode} showHomeButton={true} />
      <main className="p-8">
        <Success isNightMode={isNightMode} id={id}/>
      </main>
    </div>
  );
}
