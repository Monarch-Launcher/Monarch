import * as React from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import styled from 'styled-components';
import Page from '@_ui/page';
import Button from '@_ui/button';

const Container = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 60vh;
`;

const InfoText = styled.p`
  max-width: 40vw;
  text-align: center;
`;

const NotFound = () => {
  const navigate = useNavigate();
  const { pathname } = useLocation();

  const handleClick = React.useCallback(() => {
    navigate('/');
  }, [navigate]);

  return (
    <Page hideMenu>
      <Container>
        <h1>Page not found</h1>
        <InfoText>There is no page at {pathname}.</InfoText>
        <InfoText>
          If you got here, that means there is a bug. Please let us know how you
          got here so we can fix it 🙂
        </InfoText>
        <Button type="button" variant="primary" onClick={handleClick}>
          Go to home page
        </Button>
      </Container>
    </Page>
  );
};

export default NotFound;
