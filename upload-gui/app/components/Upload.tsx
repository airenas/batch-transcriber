"use client";
import { Button, Input, Progress } from '@nextui-org/react';
import { useTheme } from 'next-themes';
import { useRouter } from 'next/navigation';
import React, { ChangeEvent, FormEvent, useEffect, useState } from 'react';
import { toast } from 'react-toastify';

interface UploadProps {
}

const Upload: React.FC<UploadProps> = ({ }) => {
  const { theme } = useTheme();
  const [name, setName] = useState<string>('');
  const [office, setOffice] = useState<string>('');
  const [speakers, setSpeakers] = useState<string>('');
  const [audioFile, setAudioFile] = useState<File | null>(null);
  const router = useRouter();
  const [errors, setErrors] = useState<{ [key: string]: string }>({});
  const [fileSize, setFileSize] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [errName, setErrName] = useState<string>('');

  useEffect(() => {
    const savedName = localStorage.getItem('form-name');
    const savedOffice = localStorage.getItem('form-office');
    const savedSpeakerCount = localStorage.getItem('form-speakerCount');

    if (savedName) setName(savedName);
    if (savedOffice) setOffice(savedOffice);
    if (savedSpeakerCount) setSpeakers(savedSpeakerCount);
  }, []);

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      const file = e.target.files[0];
      setAudioFile(file);
      const sizeInMB = (file.size / (1024 * 1024)).toFixed(2); // Convert bytes to megabytes
      setFileSize(`${sizeInMB} MB`);
    }
  };

  const validateForm = () => {
    checkName(name);
    checkOffice(office);
    checkSpeakers(speakers);
    checkAudio(audioFile);
    console.log('Errors:', errors);
    return !errName;
  };

  const setNameLocal = (value: string) => {
    setName(value);
    localStorage.setItem('form-name', value);
    checkName(value);
  }

  const checkName = (value: string) => {
    if (!isValidStr(value)) {
      setErrName('Įveskite vardą');
    } else {
      setErrName(undefined);
    }
  };

  const checkSpeakers = (value: string) => {
    if (!value || Number(value) < 1) {
      setErrors({ ...errors, speakers: 'Nurodykite kalbėtojų kiekį' });
    } else {
      setErrors({ ...errors, speakers: undefined });
    }
  };

  const checkAudio = (value: File) => {
    if (!value) {
      setErrors({ ...errors, audioFile: 'Pasirinkite failą' });
    } else {
      setErrors({ ...errors, audioFile: undefined });
    }
  };

  const checkOffice = (value: string) => {
    if (!isValidStr(value)) {
      setErrors({ ...errors, office: 'Įveskite komisariato pavadinimą' });
    } else {
      setErrors({ ...errors, office: undefined });
    }
  };

  const setOfficeLocal = (value: string) => {
    setOffice(value);
    localStorage.setItem('form-office', value);
    checkOffice(value);
  }

  const setSpeakersLocal = (value: string) => {
    setSpeakers(value);
    localStorage.setItem('form-speakerCount', value.toString());
  }

  const isValidStr = (v: string): boolean => {
    return v && v.trim().length > 0;
  };

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    console.log('Submitting form...');
    try {

      if (!validateForm()) {
        toast.error('Užpildykite laukus', {
          theme: theme
        });
        return;
      }
      const formData = new FormData();
      formData.append('name', name);
      formData.append('office', office);
      formData.append('speakers', speakers);
      formData.append('file', audioFile);

      fetch('http://localhost:8001/upload', {
        method: 'POST',
        body: formData,
      }).then(response => {
        if (!response.ok) {
          return response.text().then(errorText => {
            if (response.status === 400) {
              const errSr = mapErr(errorText);
              throw new Error(errSr);
            }
            throw new Error(`HTTP Klaida: ${response.status} - ${errorText}`);
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
          toast.error('Klaida siunčiant: ' + error.message);
        }).finally(() => {
          setIsLoading(false);
        });

    } finally {
      console.log('exit');
      setIsLoading(false);
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
    >
      <div className="mb-4">
        <Input
          value={name}
          type="text"
          label="Vardas" placeholder="Vardas Pavardė"
          variant="bordered"
          isInvalid={errName !== undefined}
          color={errName !== undefined ? "danger" : "primary"}
          errorMessage={errName}
          onValueChange={setNameLocal}
          className="max-w-xs"
        />
      </div>
      <div className="mb-4">
        <Input
          value={office}
          type="text"
          label="Komisariatas" placeholder="Komisariatas"
          variant="bordered"
          isInvalid={errors.office !== undefined}
          color={errors.office !== undefined ? "danger" : "primary"}
          errorMessage={errors.office}
          onValueChange={setOfficeLocal}
          className="max-w-xs"
        />
      </div>
      <div className="mb-4">
        <Input
          value={speakers}
          type="number"
          label="Kalbėtojų kiekis" placeholder=""
          variant="bordered"
          isInvalid={errors.speakers !== undefined}
          color={errors.speakers !== undefined ? "danger" : "primary"}
          errorMessage={errors.speakers}
          onValueChange={setSpeakersLocal}
          className="max-w-xs"
        />
      </div>
      <div className="mb-4">
        <Input
          type="file"
          label="Audio failas"
          variant="bordered"
          accept=".mp3,.wav,.m4a"
          isInvalid={errors.audioFile !== undefined}
          color={errors.audioFile !== undefined ? "danger" : "primary"}
          errorMessage={errors.audioFile}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => handleFileChange(e)}
          className="max-w-xs"
        />
        {fileSize && <p className="text-gray-600 text-sm mt-1">Failo dydis: {fileSize}</p>}
      </div>
      <div>
        {isLoading &&
          <Progress
            size="sm"
            isIndeterminate
            aria-label="Loading..."
            className="max-w-md"
          />}
        {!isLoading &&
          <Button
            type="submit"
            color='primary'
          >
            Siųsti
          </Button>
        }
      </div>
    </form>
  );
};

export default Upload;
function mapErr(errorText: string): string {
  if (errorText === 'audio expected') {
    return 'Blogas failas - ne audio failas';
  }
  return errorText;
}

