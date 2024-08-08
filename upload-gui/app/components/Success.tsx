"use client";

import { useTheme } from 'next-themes';
import { useSearchParams } from 'next/navigation';
import React from 'react';


interface SuccessProps {
}

const Success: React.FC<SuccessProps> = ({ }) => {
  const { theme } = useTheme();
  const searchParams = useSearchParams();
  const id = searchParams.get('id') || '';

  return (
    <div className={`${theme}`}>
      {theme}
      <h1 className="text-2xl font-bold text-green-500">Audio submitted successfully</h1>
      {id && <h2 className="text-xl font-bold text-green-500">Filename: {id}</h2>}
    </div>
  );
};

export default Success;

