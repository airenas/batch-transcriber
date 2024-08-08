"use client"

import { Tooltip } from "@nextui-org/tooltip";
import { useTheme } from "next-themes";
import { useRouter } from 'next/navigation';
import React from 'react';
import { MdDarkMode, MdLightMode, MdOutlineAddCircleOutline } from "react-icons/md";
import { isDark } from "../utils";

interface HeaderProps {
  showHomeButton: boolean;
}

const Header: React.FC<HeaderProps> = ({ showHomeButton }) => {
  const { theme, setTheme } = useTheme()
  const router = useRouter();

  const handleHomeClick = () => {
    router.push('/');
  };

  const toggleNightMode = () => {
    const nextTheme = isDark(theme) ? "light" : "dark";
    setTheme(nextTheme);
  };

  return (
    <header className={`p-4 flex justify-end ${isDark(theme) ? "dark" : "light" } text-white`}>
      <div className="flex items-center space-x-4">
        {showHomeButton && (
          <Tooltip content="Siųsti naują failą">
            <button
              onClick={handleHomeClick}
            >
              <MdOutlineAddCircleOutline className={`w-6 h-6`} />
            </button>
          </Tooltip>
        )}
        <Tooltip content="Pakeisti temą">
          <button
            onClick={toggleNightMode}
          >
            {isDark(theme) ? <MdLightMode className="w-6 h-6" /> : <MdDarkMode className="w-6 h-6" />}
          </button>
        </Tooltip>
      </div>
    </header>
  );
};

export default Header;