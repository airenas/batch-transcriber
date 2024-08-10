import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { CenteredFlexBox, FullSizeCenteredFlexBox } from '@/components/styled';
import { messages } from '@/config';

function NotFound() {
  return (
    <Container sx={{ height: '100%' }}>
      <FullSizeCenteredFlexBox flexDirection="column">
        <CenteredFlexBox flexDirection="column">
          <Typography sx={{ mt: 2 }} variant="h4" color="error">
            404 Nerasta
          </Typography>
          <Divider variant="middle" />
          <Typography variant="h4" sx={{ color: (theme) => theme.palette.info.main }}>
            {messages[404]}
          </Typography>
        </CenteredFlexBox>
      </FullSizeCenteredFlexBox>
    </Container>
  );
}

export default NotFound;
