"use client";

import React from 'react';

interface SuccessProps {
  isNightMode: boolean;
  id: string;
}

const Success: React.FC<SuccessProps> = ({ isNightMode, id }) => {

  return (
    <div>
      <h1 className="text-2xl font-bold text-green-500">Audio submitted successfully</h1>
      <h2 className="text-xl font-bold text-green-500">Filename: {id}</h2>
    </div>
  );
};

export default Success;

