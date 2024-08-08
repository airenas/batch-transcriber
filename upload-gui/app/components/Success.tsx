"use client";

import { Spacer } from '@nextui-org/react';
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
    <div>
      <h1 className="text-2xl font-bold">Audio nusiųstas</h1>
      {id && <div>
        <Spacer y={4} />
        <h2 className="text-xl font-bold">Failas: {id}</h2>
      </div>
      }
    </div>
  );
};

export default Success;

