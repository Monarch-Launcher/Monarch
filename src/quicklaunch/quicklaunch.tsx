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
  const [focusedIndex, setFocusedIndex] = React.useState(-1); // Start with no game focused

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchTerm(e.target.value);
      setFocusedIndex(-1); // Clear focus when typing
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

  const handleKeyDown = React.useCallback((event: React.KeyboardEvent) => {
    if (filteredLibrary.length === 0) return;

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        setFocusedIndex((prev) =>
          prev < filteredLibrary.length - 1 ? prev + 1 : 0,
        );
        break;
      case 'ArrowUp':
        event.preventDefault();
        setFocusedIndex((prev) =>
          prev > 0 ? prev - 1 : filteredLibrary.length - 1,
        );
        break;
      default:
        break;
    }
  }, [filteredLibrary.length]);

  // Don't reset focus when filtered library changes - let user control it

  return (
    <Container onKeyDown={handleKeyDown} tabIndex={-1}>
      <Searchbar
        value={searchTerm}
        onChange={handleChange}
        placeholder="Search game..."
        autoFocus
      />
      <GamesContainer>
        {filteredLibrary.map((game, index) => (
          <GameButton
            key={game.id}
            game={game}
            isFocused={index === focusedIndex}
            onFocus={() => setFocusedIndex(index)}
          />
        ))}
      </GamesContainer>
    </Container>
  );
};

export default QuickLaunch;
