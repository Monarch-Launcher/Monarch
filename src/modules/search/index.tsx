import * as React from 'react';
import styled from 'styled-components';
import { invoke } from '@tauri-apps/api';
import Page from '../../common/page';
import GameCard from '../../common/gameCard';
import SearchBar from '../../common/searchBar';

const ResultsContainer = styled.div`
  width: 85%;
  height: calc(100% - 10rem);
  overflow-y: auto;
  border-radius: 0.5rem;
  margin: 1rem 0;
`;

type MonarchGame = {
  id: number;
  executable_path: string;
  name: string;
  platform: string;
  thumbnail_path: string;
};

const Search = () => {
  const [searchString, setSearchString] = React.useState<string>('');
  const [games, setGames] = React.useState<MonarchGame[]>([]);
  const [noResults, setNoResults] = React.useState(false);

  const searchGames = React.useCallback(async () => {
    // Return early if searchString is empty
    if (!searchString) {
      return;
    }

    const result: MonarchGame[] = await invoke('search_games', {
      name: searchString,
    });
    if (result.length === 0) {
      setNoResults(true);
    }
    setGames(result);
  }, [searchString]);

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setNoResults(false);
      setSearchString(e.target.value);
    },
    [],
  );

  return (
    <Page title="Search">
      <SearchBar
        value={searchString}
        onChange={handleChange}
        onSearchClick={searchGames}
      />
      <ResultsContainer>
        {games.map((game) => (
          <GameCard key={game.id} {...game} />
        ))}
        {noResults && (
          <p>Couldn&apos;t find any games for &quot;{searchString}&quot;</p>
        )}
      </ResultsContainer>
    </Page>
  );
};

export default Search;
