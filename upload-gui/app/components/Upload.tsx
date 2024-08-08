"use client";
import { useTheme } from 'next-themes';
import { useRouter } from 'next/navigation';
import React, { ChangeEvent, FormEvent, useEffect, useState } from 'react';
import { toast } from 'react-toastify';
import { isDark } from '../utils';

interface UploadProps {
}

const Upload: React.FC<UploadProps> = ({ }) => {
  const { theme } = useTheme();
  const [name, setName] = useState<string>('');
  const [office, setOffice] = useState<string>('');
  const [speakerCount, setSpeakerCount] = useState<number | ''>('');
  const [audioFile, setAudioFile] = useState<File | null>(null);
  const router = useRouter();
  const [errors, setErrors] = useState<{ [key: string]: string }>({});
  const [fileSize, setFileSize] = useState<string | null>(null);

  useEffect(() => {
    const savedName = localStorage.getItem('form-name');
    const savedOffice = localStorage.getItem('form-office');
    const savedSpeakerCount = localStorage.getItem('form-speakerCount');

    if (savedName) setName(savedName);
    if (savedOffice) setOffice(savedOffice);
    if (savedSpeakerCount) setSpeakerCount(Number(savedSpeakerCount));
  }, []);

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      const file = e.target.files[0];
      setAudioFile(file);

      // Convert file size to a human-readable format
      const sizeInMB = (file.size / (1024 * 1024)).toFixed(2); // Convert bytes to megabytes
      setFileSize(`${sizeInMB} MB`);
    }
  };

  const validateForm = () => {
    const newErrors: { [key: string]: string } = {};
    if (!name) newErrors.name = 'Name is required';
    if (!office) newErrors.office = 'Office is required';
    if (speakerCount === '') newErrors.speakerCount = 'Speaker count is required';
    if (!audioFile) newErrors.audioFile = 'Audio file is required';
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const setNameLocal = (value: string) => {
    setName(value);
    localStorage.setItem('form-name', value);
  }

  const setOfficeLocal = (value: string) => {
    setOffice(value);
    localStorage.setItem('form-office', value);
  }

  const setSpeakerCountLocal = (value: number) => {
    setSpeakerCount(value);
    localStorage.setItem('form-speakerCount', value.toString());
  }

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();

    if (!validateForm()) {
      toast.error('Please fill in all fields and select an audio file.');
      return;
    }

    const formData = new FormData();
    formData.append('name', name);
    formData.append('office', office);
    formData.append('speakers', speakerCount.toString());
    formData.append('file', audioFile);

    fetch('http://localhost:8001/upload', {
      method: 'POST',
      body: formData,
    }).then(response => {
      if (!response.ok) {
        return response.text().then(errorText => {
          throw new Error(`HTTP Error: ${response.status} - ${errorText}`);
        });
      }
      return response.json(); // Proceed with parsing JSON if status is OK
    })
      .then(data => {
        console.log('Form submitted:', data);
        router.push('/success?id=' + data.id);
      })
      .catch(error => {
        console.error('Error submitting form:', error);
        toast.error('Error submitting form: ' + error.message);
      });
  };

  return (
    <form
      onSubmit={handleSubmit}
      className={`p-6 shadow-md rounded ${isDark(theme) ? 'bg-gray-800 text-white border-gray-600' : 'bg-white text-black border-gray-300'} border`}
    >
      <div className="mb-4">
        <label htmlFor="name" className="block text-sm font-medium mb-2">Name</label>
        <input
          type="text"
          id="name"
          value={name}
          onChange={(e) => setNameLocal(e.target.value)}
          className={`w-full p-2 border rounded ${isDark(theme) ? 'bg-gray-700 text-white' : 'bg-gray-100 text-black'}`}
        />
        {errors.name && <p className="text-red-500 text-sm mt-1">{errors.name}</p>}
      </div>
      <div className="mb-4">
        <label htmlFor="office" className="block text-sm font-medium mb-2">Office</label>
        <input
          type="text"
          id="office"
          value={office}
          onChange={(e) => setOfficeLocal(e.target.value)}
          className={`w-full p-2 border rounded ${isDark(theme) ? 'bg-gray-700 text-white' : 'bg-gray-100 text-black'}`}
        />
        {errors.office && <p className="text-red-500 text-sm mt-1">{errors.office}</p>}
      </div>
      <div className="mb-4">
        <label htmlFor="speakerCount" className="block text-sm font-medium mb-2">Speaker Count</label>
        <input
          type="number"
          id="speakerCount"
          value={speakerCount}
          onChange={(e) => setSpeakerCountLocal(Number(e.target.value))}
          className={`w-full p-2 border rounded ${isDark(theme) ? 'bg-gray-700 text-white' : 'bg-gray-100 text-black'}`}
        />
        {errors.speakerCount && <p className="text-red-500 text-sm mt-1">{errors.speakerCount}</p>}
      </div>
      <div className="mb-4">
        <label htmlFor="audioFile" className="block text-sm font-medium mb-2">Audio File</label>
        <input
          type="file"
          id="audioFile"
          onChange={handleFileChange}
          className={`w-full p-2 border rounded ${isDark(theme) ? 'bg-gray-700 text-white' : 'bg-gray-100 text-black'}`}
        />
        {fileSize && <p className="text-gray-600 text-sm mt-1">File size: {fileSize}</p>}
        {errors.audioFile && <p className="text-red-500 text-sm mt-1">{errors.audioFile}</p>}
      </div>
      <button
        type="submit"
        className={`w-full py-2 rounded ${isDark(theme) ? 'bg-blue-500 hover:bg-blue-600' : 'bg-blue-700 hover:bg-blue-800'} text-white`}
      >
        Submit
      </button>
    </form>
  );
};

export default Upload;
