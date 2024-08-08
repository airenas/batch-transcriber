"use client";
import { Button, Input, Progress, Spacer } from '@nextui-org/react';
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
  const [fileSize, setFileSize] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  const [errName, setErrName] = useState<string | null>(null);
  const [errOffice, setErrOffice] = useState<string | null>(null);
  const [errSpeakers, setErrSpeakers] = useState<string | null>(null);
  const [errAudioFile, setErrAudioFile] = useState<string | null>(null);


  useEffect(() => {
    const savedName = localStorage.getItem('form-name');
    const savedOffice = localStorage.getItem('form-office');
    const savedSpeakerCount = localStorage.getItem('form-speakerCount');

    if (savedName) setName(savedName);
    if (savedOffice) setOffice(savedOffice);
    if (savedSpeakerCount) setSpeakers(savedSpeakerCount);
  }, []);

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    var file: File | null = null;
    if (e.target.files && e.target.files[0]) {
      file = e.target.files[0];
      setAudioFile(file);
      const sizeInMB = (file.size / (1024 * 1024)).toFixed(2); // Convert bytes to megabytes
      setFileSize(`${sizeInMB} MB`);
    }
    checkAudio(file);
  };

  const validateForm = () => {
    checkName(name, true);
    checkOffice(office, true);
    checkSpeakers(speakers, true);
    checkAudio(audioFile, true);
    return errName === null && errOffice === null && errSpeakers === null && errAudioFile === null;
  };

  const setNameLocal = (value: string) => {
    setName(value);
    localStorage.setItem('form-name', value);
    checkName(value);
  }

  const checkName = (value: string, fail: boolean = false) => {
    if (!isValidStr(value)) {
      if (fail) {
        setErrName('Įveskite vardą');
      }
    } else {
      setErrName(null);
    }
  };

  const checkSpeakers = (value: string, fail: boolean = false) => {
    if (!value || Number(value) < 1) {
      if (fail) {
        setErrSpeakers('Nurodykite kalbėtojų kiekį');
      }
    } else {
      setErrSpeakers(null);
    }
  };

  const checkAudio = (value: File, fail: boolean = false) => {
    if (!value) {
      if (fail) {
        setErrAudioFile('Pasirinkite failą');
      }
    } else {
      setErrAudioFile(null);
    }
  };

  const checkOffice = (value: string, fail: boolean = false) => {
    if (!isValidStr(value)) {
      if (fail) {
        setErrOffice('Įveskite komisariato pavadinimą');
      }
    } else {
      setErrOffice(null);
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
    checkSpeakers(value);
  }

  const isValidStr = (v: string): boolean => {
    return v && v.trim().length > 0;
  };

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();

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

    setIsLoading(true);
    fetch('http://localhost:8001/upload', { ///TODO
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
      return response.json();
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
  };

  return (
    <form
      onSubmit={handleSubmit}
    >
      <div className="mb-4">
        <Input
          isRequired
          value={name}
          type="text"
          label="Vardas Pavardė"
          variant="bordered"
          isInvalid={errName !== null}
          errorMessage={errName}
          onValueChange={setNameLocal}
          className="max-w-xs"
          size='lg'
        />
      </div>
      <div className="mb-4">
        <Input
          isRequired
          value={office}
          type="text"
          label="Komisariatas"
          variant="bordered"
          isInvalid={errOffice !== null}
          errorMessage={errOffice}
          onValueChange={setOfficeLocal}
          className="max-w-xs"
          size='lg'
        />
      </div>
      <div className="mb-4">
        <Input
          isRequired
          value={speakers}
          type="number"
          label="Kalbėtojų kiekis"
          variant="bordered"
          isInvalid={errSpeakers !== null}
          errorMessage={errSpeakers}
          onValueChange={setSpeakersLocal}
          className="max-w-xs"
          size='lg'
        />
      </div>
      <div>
        <Spacer y={10} />
      </div>
      <div className="mb-4">
        <Input
          isRequired
          type="file"
          description={fileSize ? `Failo dydis: ${fileSize}` : null}
          label="Audio failas"
          variant="bordered"
          accept=".mp3,.wav,.m4a"
          isInvalid={errAudioFile !== null}
          errorMessage={errAudioFile}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => handleFileChange(e)}
          className="max-w-xs"
          size='lg'
        />
      </div>
      <div>
        {isLoading &&
          <Progress
            size="lg"
            isIndeterminate
            aria-label="Loading..."
            className="max-w-md"
          />}
        {!isLoading &&
          <Button
            type="submit"
            color='primary'
            size='lg'
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

