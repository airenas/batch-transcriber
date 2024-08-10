import Typography from '@mui/material/Typography';

import Meta from '@/components/Meta';
import { FullSizeCenteredFlexBox } from '@/components/styled';

function About() {
  return (
    <>
      <Meta title="apie" />
      <FullSizeCenteredFlexBox>
        <Typography variant="h2">Di Polis Audio saugykla</Typography>
      </FullSizeCenteredFlexBox>
    </>
  );
}

export default About;
