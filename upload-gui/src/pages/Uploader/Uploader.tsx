import Meta from '@/components/Meta';
import { FullSizeCenteredFlexBox } from '@/components/styled';
import { makeLink, serverUrl } from '@/config';
import useNotifications from '@/store/notifications';
import { Box, Button, LinearProgress, Stack, TextField } from '@mui/material';
import { ChangeEvent, FormEvent, useEffect, useState } from 'react';
import { useNavigate } from "react-router-dom";

function Uploader() {
  const [, notificationsActions] = useNotifications();
  const [name, setName] = useState<string>('');
  const [nameError, setNameError] = useState(false);
  const [office, setOffice] = useState<string>('');
  const [officeError, setOfficeError] = useState(false);
  const [speakers, setSpeakers] = useState<number>(0);
  const [speakersError, setSpeakersError] = useState(false);
  const [file, setFile] = useState<File | null>(null);
  const [fileError, setFileError] = useState(false);
  const [fileSize, setFileSize] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const navigate = useNavigate();

  useEffect(() => {
    const savedName = localStorage.getItem('form-name');
    const savedOffice = localStorage.getItem('form-office');
    const savedSpeakerCount = localStorage.getItem('form-speakers');

    if (savedName) setName(savedName);
    if (savedOffice) setOffice(savedOffice);
    if (savedSpeakerCount) setSpeakers(Number(savedSpeakerCount));
  }, []);

  function validateForm(): boolean {
    setNameError(name === '');
    setOfficeError(office === '');
    setFileError(file === null);
    setSpeakersError(speakers < 1);
    return name !== '' && office !== '' && file !== null && speakers > 0;
  }

  const handleNameChange = (e: ChangeEvent<HTMLInputElement>) => {
    setName(e.target.value);
    localStorage.setItem('form-name', e.target.value);
    setNameError(e.target.value === '')
  };

  const handleOfficeChange = (e: ChangeEvent<HTMLInputElement>) => {
    setOffice(e.target.value);
    localStorage.setItem('form-office', e.target.value);
    setOfficeError(e.target.value === '')
  }

  const handleSpeakersChange = (e: ChangeEvent<HTMLInputElement>) => {
    setSpeakers(Number(e.target.value));
    localStorage.setItem('form-speakers', e.target.value);
    setSpeakersError(Number(e.target.value) < 1)
  }

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    let _file: File | null = null;
    if (e.target.files && e.target.files[0]) {
      _file = e.target.files[0];
      setFile(_file);
      const sizeInMB = (_file.size / (1024 * 1024)).toFixed(2); // Convert bytes to megabytes
      setFileSize(`${sizeInMB} MB`);
      setFileError(false);
    } else {
      setFileSize('');
      setFile(null);
      setFileError(true);
    }
  };

  function showError(msg: string) {
    notificationsActions.push({
      options: {
        variant: 'errorNotification',
      },
      message: msg,
    });
  };

  function showInfo(msg: string) {
    notificationsActions.push({
      options: {
        variant: 'infoNotification',
      },
      message: msg,
    });
  };

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!validateForm()) {
      showError('Užpildykite laukus');
      return;
    }
    const formData = new FormData();
    formData.append('name', name);
    formData.append('office', office);
    formData.append('speakers', speakers.toString());
    if (file) {
      formData.append('file', file);
    }

    setIsLoading(true);
    console.log('Submitting form:', serverUrl);
    fetch(serverUrl, { ///TODO
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
        showInfo('Audio išsaugotas');
        navigate(makeLink('/success?id=' + data.id));
      })
      .catch(error => {
        console.error('Error submitting form:', error);
        showError('Klaida siunčiant: ' + error.message);
      }).finally(() => {
        setIsLoading(false);
      });
  };

  return (
    <>
      <Meta title="siųsti failą" />
      <FullSizeCenteredFlexBox>
        <Box component="form" onSubmit={handleSubmit} noValidate sx={{ width: '100%', maxWidth: "500px" }} >
          <Stack spacing={2}>
            <TextField
              label="Vardas Pavardė"
              required
              id="name-input"
              value={name}
              type="text"
              onChange={handleNameChange}
              error={nameError}
              helperText={nameError ? "Įveskite vardą ir pavardę" : ""}
            />

            <TextField
              label="Komisariatas"
              required
              id="office-input"
              value={office}
              type="text"
              onChange={handleOfficeChange}
              error={officeError}
              helperText={officeError ? "Įveskite policijos komisariatą" : ""}
            />

            <TextField
              label="Kalbėtojų kiekis"
              required
              id="speakers-input"
              value={speakers}
              type="number"
              onChange={handleSpeakersChange}
              error={speakersError}
              helperText={speakersError ? "Nurodykite kalbėtojų kiekį audio faile" : ""}
            />

            <TextField type="file"
              label={file ? "Audio failas" : ""}
              inputProps={{
                accept: ".mp3,.wav,.m4a",
                multiple: false
              }}
              required
              id="file-input"
              error={fileError}
              helperText={fileError ? "Pasirinkite failą" : (fileSize ? `Failo dydis: ${fileSize}` : "")}
              onChange={handleFileChange}
            />

            <Box sx={{ height: '50px' }} >
              {!isLoading && <Button variant="contained" color="primary" type="submit"
              >Siųsti</Button>}
              {isLoading && <LinearProgress sx={{ top: "20px" }} />}
            </Box>
          </Stack>
        </Box>
      </FullSizeCenteredFlexBox>
    </>
  );
}

export default Uploader;
function mapErr(errorText: string): string {
  if (errorText === 'audio expected') {
    return 'Blogas failas - ne audio failas';
  }
  return errorText;
}

