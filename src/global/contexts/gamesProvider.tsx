import * as React from 'react';

type MonarchGame = {
  id: number;
  executable_path: string;
  name: string;
  platform: string;
  thumbnail_path: string;
};

type MonarchGameContextType = {
  library: MonarchGame[];
  searchedGames: MonarchGame[];
  updateLibrary: (searchResult: MonarchGame[]) => void;
  updateSearchedGames: (searchResult: MonarchGame[]) => void;
};

const initialState = {
  library: [],
  searchedGames: [],
  updateLibrary: () => {},
  updateSearchedGames: () => {},
};

const GamesContext = React.createContext<MonarchGameContextType>(initialState);
export const useGames = () => React.useContext(GamesContext);

type Props = {
  children: React.ReactNode;
};

const GamesProvider = ({ children }: Props) => {
  const [library, setLibrary] = React.useState<MonarchGame[]>([]);
  const [searchedGames, setSearchedGames] = React.useState<MonarchGame[]>([]);

  const updateLibrary = React.useCallback((searchResult: MonarchGame[]) => {
    setLibrary([...searchResult]);
  }, []);

  const updateSearchedGames = React.useCallback(
    (searchResult: MonarchGame[]) => {
      setSearchedGames([...searchResult]);
    },
    [],
  );

  const value = React.useMemo(() => {
    return {
      library,
      searchedGames,
      updateLibrary,
      updateSearchedGames,
    };
  }, [library, searchedGames, updateLibrary, updateSearchedGames]);

  return (
    <GamesContext.Provider value={value}>{children}</GamesContext.Provider>
  );
};

export default GamesProvider;
