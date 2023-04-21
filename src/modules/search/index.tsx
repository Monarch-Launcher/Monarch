import * as React from 'react';
import styled from 'styled-components';
import Page from '../../common/page';
import GameCard from '../../common/gameCard';
import SearchBar from '../../common/searchBar';
import { useSearchGames } from '../../global/contexts/searchGamesProvider';

const ResultsContainer = styled.div`
  width: 85%;
  height: calc(100% - 10rem);
  overflow-y: auto;
  border-radius: 0.5rem;
  margin: 1rem 0;
`;

const Search = () => {
  const [searchString, setSearchString] = React.useState('');
  const [noResults, setNoResults] = React.useState(false);
  const { searchedGames, loading, error, searchGames } = useSearchGames();

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setNoResults(false);
      setSearchString(e.target.value);
    },
    [],
  );

  const handleClick = React.useCallback(async () => {
    // Return early if searchString is empty
    if (!searchString) {
      return;
    }
    await searchGames(searchString);
    setNoResults(searchedGames.length === 0);
  }, [searchGames, searchString, searchedGames]);

  return (
    <Page title="Search">
      <SearchBar
        value={searchString}
        onChange={handleChange}
        onSearchClick={handleClick}
      />
      <ResultsContainer>
        {searchedGames.map((game) => (
          <GameCard key={game.id} {...game} />
        ))}
        {noResults && (
          <p>Couldn&apos;t find any games for &quot;{searchString}&quot;</p>
        )}
        {error && <p>Something went wrong</p>}
        {loading && <p>Loading...</p>}
      </ResultsContainer>
    </Page>
  );
};

export default Search;
