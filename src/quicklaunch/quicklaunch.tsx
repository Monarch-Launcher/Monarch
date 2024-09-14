import { useLibrary } from '@global/contexts/libraryProvider';
import { MonarchGame } from '@global/types';
import * as React from 'react';
import styled from 'styled-components';

import GameButton from './_components/gameButton';

const Container = styled.div`
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
`;

const Searchbar = styled.input`
  border: none;
  border-radius: 0.3rem;
  width: 100%;
  height: 3.25rem;
  font-size: 1.5rem;
  padding: 0.2rem;
  margin-bottom: 1rem;

  &:focus {
    outline: none;
  }
`;

const GamesContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 1rem;
  height: 70vh;
  overflow-y: scroll;
`;

const QuickLaunch = () => {
  const { library } = useLibrary();
  const [searchTerm, setSearchTerm] = React.useState('');

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchTerm(e.target.value);
    },
    [],
  );

  const filteredLibrary = React.useMemo<MonarchGame[]>(() => {
    return library.filter((game) =>
      game.name
        .replace(/[.,/#!$%^&*;:{}=\-_`~()]/g, '')
        .toLowerCase()
        .match(searchTerm.toLowerCase()),
    );
  }, [library, searchTerm]);

  return (
    <Container>
      <Searchbar
        value={searchTerm}
        onChange={handleChange}
        placeholder="Search game..."
        autoFocus
      />
      <GamesContainer>
        {filteredLibrary.map((game) => (
          <GameButton key={game.id} game={game} />
        ))}
      </GamesContainer>
    </Container>
  );
};

export default QuickLaunch;
