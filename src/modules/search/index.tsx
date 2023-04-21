import * as React from 'react';
import styled from 'styled-components';
import Page from '../../common/page';
import GameCard from '../../common/gameCard';
import SearchBar from '../../common/searchBar';
import { useSearchGames } from '../../global/contexts/searchGamesProvider';
import Spinner from '../../common/spinner';
import Error from '../../common/error';

const ResultsContainer = styled.div`
  width: 85%;
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
    // Return early if searchString is empty
    if (!searchString) {
      return;
    }
    await searchGames(searchString);
  }, [searchGames, searchString]);

  return (
    <Page title="Search">
      <SearchBar
        value={searchString}
        onChange={handleChange}
        onSearchClick={handleClick}
        placeholder="Search"
      />
      <ResultsContainer>
        {loading ? (
          <Spinner />
        ) : (
          searchedGames.map((game) => <GameCard key={game.id} {...game} />)
        )}
        {!loading && results?.empty && <p>{results.message}</p>}
        {!loading && error && (
          <Error description="Couldn't load games" onRetry={handleClick} />
        )}
      </ResultsContainer>
    </Page>
  );
};

export default Search;
