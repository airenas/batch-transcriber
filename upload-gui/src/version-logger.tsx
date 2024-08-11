import React, { useEffect } from 'react';
import { VERSION } from './version';

const VersionLogger: React.FC = () => {
  useEffect(() => {
    console.log(`version: ${VERSION}`);
  }, []);

  return null;
};

export default VersionLogger;
