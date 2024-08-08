"use client"

import React from 'react';
import { useRouter } from 'next/navigation';

interface HeaderProps {
  isNightMode: boolean;
  toggleNightMode: () => void;
  showHomeButton: boolean;
}

const Header: React.FC<HeaderProps> = ({ isNightMode, toggleNightMode, showHomeButton }) => {

  const router = useRouter();

  const handleHomeClick = () => {
    router.push('/'); // Navigate to the home page
  };

  return (
    <header className={`p-4 ${isNightMode ? 'bg-gray-800' : 'bg-gray-200'} text-white`}>
      {showHomeButton && (
      <button
        onClick={handleHomeClick}
        className={`mt-2 p-2 ${isNightMode ? 'bg-blue-300' : 'bg-blue-500'} text-white rounded`}
      >
        Upload
      </button>
      )}

      <h1 className="text-3xl font-bold">My Next.js App</h1>
      <button
        onClick={toggleNightMode}
        className={`mt-2 p-2 ${isNightMode ? 'bg-blue-300' : 'bg-blue-500'} text-white rounded`}
      >
        {isNightMode ? 'Switch to Day Mode' : 'Switch to Night Mode'}
      </button>


    </header>
  );
};

export default Header;