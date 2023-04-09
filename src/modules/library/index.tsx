import * as React from 'react';
import styled from 'styled-components';
import Page from '../../common/page';
import GameCard from '../../common/gameCard';

const LibraryContainer = styled.div`
  width: 85%;
  height: calc(100% - 7rem);
  overflow-y: auto;
  border-radius: 0.5rem;
`;

const Library = () => {
  return (
    <Page title="Library">
      <p>Prob like a search bar over here</p>
      <LibraryContainer>
        <GameCard />
        <GameCard />
        <GameCard />
        <GameCard />
        <GameCard />
        <GameCard />
        <GameCard />
        <GameCard />
        <GameCard />
      </LibraryContainer>
    </Page>
  );
};

export default Library;
