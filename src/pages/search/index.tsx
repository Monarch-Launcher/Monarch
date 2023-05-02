import Error from '@_ui/error';
import GameCard from '@_ui/gameCard';
import Page from '@_ui/page';
import SearchBar from '@_ui/searchBar';
import Spinner from '@_ui/spinner';
import { useSearchGames } from '@global/contexts/searchGamesProvider';
import * as React from 'react';
import styled from 'styled-components';

const ResultsContainer = styled.div`
  width: 100%;
  height: calc(100% - 10rem);
  overflow-y: auto;
  border-radius: 0.5rem;
  margin: 1rem 0;
`;

const Search = () => {
  const [searchString, setSearchString] = React.useState('');
  const { searchedGames, loading, error, searchGames, results } =
    useSearchGames();

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchString(e.target.value);
    },
    [],
  );

  const handleClick = React.useCallback(async () => {
    // Return early if searchString is empty or the same as previous searchString
    if (!searchString || searchString === results?.searchString) {
      return;
    }
    await searchGames(searchString);
  }, [searchGames, searchString, results?.searchString]);

  return (
    <Page title="Search">
      <SearchBar
        value={searchString}
        onChange={handleChange}
        onSearchClick={handleClick}
        placeholder="Search"
        loading={loading}
      />
      <ResultsContainer>
        {loading ? (
          <Spinner />
        ) : (
          searchedGames.map((game) => (
            <GameCard
              key={game.id}
              id={game.id}
              executablePath={game.executable_path}
              platform={game.platform}
              name={game.name}
              platformId={game.platform_id}
              thumbnailPath={game.thumbnail_path}
            />
          ))
        )}
        {!loading && results?.empty && <p>{results.emptyMessage}</p>}
        {!loading && error && (
          <Error description="Couldn't load games" onRetry={handleClick} />
        )}
      </ResultsContainer>
    </Page>
  );
};

export default Search;
