import Typography from '@mui/material/Typography';

import Meta from '@/components/Meta';
import { FullSizeCenteredFlexBox } from '@/components/styled';
import { makeLink } from '@/config';
import useTheme from '@/store/theme';
import { Box, Button } from '@mui/material';
import Stack from '@mui/material/Stack';
import { useNavigate, useSearchParams } from 'react-router-dom';

function Success() {
  const [searchParams] = useSearchParams();
  const id = searchParams.get('id') || '';
  const navigate = useNavigate();
  const [theme] = useTheme();


  function navigateNew(): void {
    navigate(makeLink('/'));
  }

  return (
    <>
      <Meta title="sėkmingai nusiųsta" />
      <FullSizeCenteredFlexBox>
        <Stack spacing={2}>
          <Typography variant="h3" color={theme === 'light' ? 'primary' : 'text.secondary'} >Failas nusiųstas</Typography>

          {id && <Typography variant="h4" color={theme === 'light' ? 'primary' : 'text.secondary'}>Suteiktas vardas: {id}</Typography>}

          <Box sx={{ height: '50px' }} >
            <Button variant="contained" color='primary' onClick={navigateNew}>Siųsti naują failą</Button>
          </Box>
        </Stack>
      </FullSizeCenteredFlexBox >
    </>
  );
}

export default Success;
