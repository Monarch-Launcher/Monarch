import * as React from 'react';
import styled from 'styled-components';
import { invoke } from '@tauri-apps/api';
import Page from '../../common/page';
import GameCard from '../../common/gameCard';
import SearchBar from '../../common/searchBar';
import { useGames } from '../../global/contexts/gamesProvider';

const ResultsContainer = styled.div`
  width: 85%;
  height: calc(100% - 10rem);
  overflow-y: auto;
  border-radius: 0.5rem;
  margin: 1rem 0;
`;

export type MonarchGame = {
  id: number;
  executable_path: string;
  name: string;
  platform: string;
  thumbnail_path: string;
};

const Search = () => {
  const [searchString, setSearchString] = React.useState<string>('');
  const [noResults, setNoResults] = React.useState(false);
  const [error, setError] = React.useState(false);
  const { searchedGames, updateSearchedGames } = useGames();

  const searchGames = React.useCallback(async () => {
    // Return early if searchString is empty
    if (!searchString) {
      return;
    }

    try {
      const result: MonarchGame[] = await invoke('search_games', {
        name: searchString,
      });
      if (result.length === 0) {
        setNoResults(true);
      }
      updateSearchedGames(result);
      setError(false);
    } catch (err) {
      setError(true);
    }
  }, [searchString, updateSearchedGames]);

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
        {searchedGames.map((game) => (
          <GameCard key={game.id} {...game} />
        ))}
        {noResults && (
          <p>Couldn&apos;t find any games for &quot;{searchString}&quot;</p>
        )}
        {error && <p>Something went wrong</p>}
      </ResultsContainer>
    </Page>
  );
};

export default Search;
