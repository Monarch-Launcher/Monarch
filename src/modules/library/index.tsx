import * as React from 'react';
import styled from 'styled-components';
import Page from '../../common/page';
import GameCard from '../../common/gameCard';

const CardsContainer = styled.div`
  display: inline-block;
  width: 100%;
  max-height: 100vh;
  overflow-y: auto;
`;

const Library = () => {
  return (
    <Page title="Library">
      <p>library content</p>
      <CardsContainer>
        <GameCard />
        <GameCard />
        <GameCard />
        <GameCard />
        <GameCard />
        <GameCard />
      </CardsContainer>
    </Page>
  );
};

export default Library;
