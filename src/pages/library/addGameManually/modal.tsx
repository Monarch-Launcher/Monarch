import Button from '@_ui/button';
import Error from '@_ui/error';
import GameCard from '@_ui/gameCard';
import Modal from '@_ui/modal';
import SearchBar from '@_ui/searchBar';
import Spinner from '@_ui/spinner';
import { useLibrary } from '@global/contexts/libraryProvider';
import { useSearchGames } from '@global/contexts/searchGamesProvider';
import { AiOutlineSearch, BiArrowBack, FaFolderOpen, MdClose } from '@global/icons';
import { MonarchGame } from '@global/types';
import { Switch } from '@mantine/core';
import { invoke } from '@tauri-apps/api/core';
import * as dialog from '@tauri-apps/plugin-dialog';
import * as React from 'react';
import styled from 'styled-components';

const ModalHeaderContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: flex-start;
  width: 100%;
  color: #fff;
`;

const ModalHeader = styled.h2`
  margin: 0.5rem 0;
  color: #fff;
`;

const ModalButtons = styled.div`
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 1rem;
  margin: 2rem 0 1rem 0;
  color: #fff;
`;

const ModalHeaderButtons = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
  width: 100%;
`;

const SearchView = styled.div`
  color: #fff;
  padding-right: 1.75rem;
  height: 500px;
  display: flex;
  flex-direction: column;
`;

const SearchRow = styled.div`
  display: flex;
  align-items: center;
  margin-bottom: 1rem;
  gap: 1rem;
`;

const MonarchSwitch = styled(Switch)`
  input:checked + .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.primary};
    border-color: ${({ theme }) => theme.colors.primary};
  }

  .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.secondary};
    border-color: ${({ theme }) => theme.colors.secondary};
  }

  .mantine-Switch-label {
    color: ${({ theme }) => theme.colors.white};
  }

  &:hover {
    opacity: 0.9;
  }

  display: flex;
  align-items: center;

  label {
    margin-left: 1rem;
    user-select: none;
  }
`;

const ResultsContainer = styled.div`
  width: 100%;
  flex: 1;
  overflow-y: auto;
  border-radius: 0.5rem;
  margin: 1rem 0;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 1rem;
  padding: 0 1rem;
`;

const SearchButton = styled(Button)`
  min-width: 120px;
`;

const ModalContentContainer = styled.div`
  color: #fff;
  padding-right: 1.75rem;
`;

const FormGroup = styled.div`
  margin-bottom: 1.5rem;
`;

const Label = styled.label`
  display: block;
  margin-bottom: 0.5rem;
  color: #fff;
  font-weight: 600;
`;

const Input = styled.input`
  width: 100%;
  padding: 0.75rem;
  border: 2px solid ${({ theme }) => theme.colors.secondary};
  border-radius: 0.5rem;
  background-color: ${({ theme }) => theme.colors.black};
  color: #fff;
  font-size: 1rem;

  &:focus {
    outline: none;
    border-color: #fa5002;
  }

  &::placeholder {
    color: ${({ theme }) => theme.colors.secondary};
  }
`;

const TextArea = styled.textarea`
  width: 100%;
  padding: 0.75rem;
  border: 2px solid ${({ theme }) => theme.colors.secondary};
  border-radius: 0.5rem;
  background-color: ${({ theme }) => theme.colors.black};
  color: #fff;
  font-size: 1rem;
  min-height: 80px;
  resize: vertical;
  font-family: inherit;

  &:focus {
    outline: none;
    border-color: #fa5002;
  }

  &::placeholder {
    color: ${({ theme }) => theme.colors.secondary};
  }
`;

const ErrorText = styled.p`
  margin: 0.2rem 0 0 0;
  color: ${({ theme }) => theme.colors.error};
`;

const InputGroup = styled.div`
  display: flex;
  gap: 0.5rem;
  align-items: stretch;
  margin-right: -1.75rem;
`;

const InputWithButton = styled(Input)`
  flex: 1;
`;

const BrowseButton = styled(Button)`
  flex-shrink: 0;
  padding: 0.75rem 1rem;
`;

type Props = {
  opened: boolean;
  close: () => void;
  selectedFilePath?: string;
  onGameAdded?: () => void;
};

export default ({ opened, close, selectedFilePath, onGameAdded }: Props) => {
  const { addGameToLibrary } = useLibrary();
  const {
    searchedGames,
    loading,
    error: searchError,
    searchGames,
    results,
    clearSearchResults,
  } = useSearchGames();
  const [gameName, setGameName] = React.useState('');
  const [thumbnailPath, setThumbnailPath] = React.useState('');
  const [errorMessage, setErrorMessage] = React.useState<string | undefined>();
  const [showSearchView, setShowSearchView] = React.useState(false);
  const [searchString, setSearchString] = React.useState('');
  const [searchOnMonarch, setSearchOnMonarch] = React.useState(true);

  const handleGameNameChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setGameName(e.target.value);
      setErrorMessage(undefined);
    },
    [],
  );

  const handleThumbnailPathChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setThumbnailPath(e.target.value);
      setErrorMessage(undefined);
    },
    [],
  );

  const handleBrowseThumbnail = React.useCallback(async () => {
    try {
      const selected = await dialog.open({
        multiple: false,
        title: 'Choose a thumbnail image',
        directory: false,
        filters: [
          {
            name: 'Image Files',
            extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp', 'svg'],
          },
          {
            name: 'All Files',
            extensions: ['*'],
          },
        ],
      });

      if (selected) {
        setThumbnailPath(selected as string);
        setErrorMessage(undefined);
      }
    } catch (error) {
      setErrorMessage('Failed to open file dialog.');
    }
  }, []);

  const handleAddGame = React.useCallback(async () => {
    if (gameName.length === 0) {
      setErrorMessage('Game name must be at least 1 character.');
      return;
    }

    if (!selectedFilePath) {
      setErrorMessage('No file selected.');
      return;
    }

    const game: MonarchGame = {
      id: '',
      platform_id: '',
      executable_path: selectedFilePath,
      name: gameName,
      platform: 'monarch-binary',
      thumbnail_path: thumbnailPath,
      store_page: '',
      compatibility: '',
      launch_args: '',
    };

    // Add game to frontend library immediately for instant feedback
    addGameToLibrary(game);

    await invoke('manual_add_game', {
      game,
    });

    // Refresh the library after adding the game
    if (onGameAdded) {
      onGameAdded();
    }

    close();
    setGameName('');
    setThumbnailPath('');
    setErrorMessage(undefined);
    clearSearchResults();
  }, [
    close,
    gameName,
    selectedFilePath,
    thumbnailPath,
    onGameAdded,
    addGameToLibrary,
    clearSearchResults,
  ]);

  const handleCancel = React.useCallback(() => {
    close();
    setGameName('');
    setThumbnailPath('');
    setErrorMessage(undefined);
    setShowSearchView(false);
    setSearchString('');
    clearSearchResults();
  }, [close, clearSearchResults]);

  const handleSearchClick = React.useCallback(() => {
    setShowSearchView(true);
    clearSearchResults();
  }, [clearSearchResults]);

  const handleBackToManual = React.useCallback(() => {
    setShowSearchView(false);
  }, []);

  const handleSearchStringChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchString(e.target.value);
    },
    [],
  );

  const handleSwitchChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchOnMonarch(e.target.checked);
    },
    [],
  );

  const handleSearchSubmit = React.useCallback(async () => {
    if (!searchString) {
      return;
    }
    await searchGames(searchString, searchOnMonarch);
  }, [searchGames, searchString, searchOnMonarch]);

  const handleGameSelect = React.useCallback((game: MonarchGame) => {
    // Pre-fill the form with selected game data
    setGameName(game.name);
    setThumbnailPath(game.thumbnail_path || '');
    setShowSearchView(false);
  }, []);

  const modalHeader = React.useMemo<JSX.Element>(() => {
    return (
      <ModalHeaderContainer>
        <ModalHeaderButtons>
          {showSearchView ? (
            <>
              <ModalHeader>Search Games</ModalHeader>
              <Button
                type="button"
                variant="secondary"
                onClick={handleBackToManual}
                leftIcon={BiArrowBack}
              >
                Back to Manual
              </Button>
            </>
          ) : (
            <>
              <ModalHeader>Add Game Manually</ModalHeader>
              <SearchButton
                type="button"
                variant="secondary"
                onClick={handleSearchClick}
                leftIcon={AiOutlineSearch}
              >
                Search Games
              </SearchButton>
            </>
          )}
        </ModalHeaderButtons>
      </ModalHeaderContainer>
    );
  }, [showSearchView, handleSearchClick, handleBackToManual]);

  return (
    <Modal
      title={modalHeader}
      opened={opened}
      onClose={handleCancel}
      centered
      withCloseButton={false}
      size="900px"
    >
      {showSearchView ? (
        <>
          <SearchView>
            <SearchRow>
              <SearchBar
                value={searchString}
                onChange={handleSearchStringChange}
                onSearchClick={handleSearchSubmit}
                placeholder="Search for games"
                loading={loading}
                fullWidth
              />
              <MonarchSwitch
                checked={searchOnMonarch}
                onChange={handleSwitchChange}
                size="md"
                label="Search on monarch-launcher.com"
                labelPosition="right"
              />
            </SearchRow>
            <ResultsContainer>
              {loading ? (
                <Spinner />
              ) : (
                searchedGames.map((game) => (
                  <div
                    key={game.id}
                    onClick={() => handleGameSelect(game)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        handleGameSelect(game);
                      }
                    }}
                    role="button"
                    tabIndex={0}
                    style={{ cursor: 'pointer' }}
                  >
                    <GameCard
                      id={game.id}
                      executablePath={game.executable_path}
                      platform={game.platform}
                      name={game.name}
                      platformId={game.platform_id}
                      thumbnailPath={game.thumbnail_path}
                      storePage={game.store_page}
                      hideDownload
                    />
                  </div>
                ))
              )}
              {!loading && results?.empty && <p>{results.emptyMessage}</p>}
              {!loading && searchError && (
                <Error description="Couldn't load games" onRetry={handleSearchSubmit} />
              )}
            </ResultsContainer>
          </SearchView>
          <ModalButtons>
            <Button
              type="button"
              variant="secondary"
              onClick={handleCancel}
              leftIcon={MdClose}
            >
              Cancel
            </Button>
          </ModalButtons>
        </>
      ) : (
        <>
          <ModalContentContainer>
            <FormGroup>
              <Label htmlFor="gameName">Game Name</Label>
              <Input
                id="gameName"
                type="text"
                value={gameName}
                onChange={handleGameNameChange}
                placeholder="Enter game name"
                autoFocus
                maxLength={100}
              />
            </FormGroup>

            <FormGroup>
              <Label htmlFor="filePath">Executable Path</Label>
              <TextArea
                id="filePath"
                value={selectedFilePath || ''}
                placeholder="No file selected"
                readOnly
              />
            </FormGroup>

            <FormGroup>
              <Label htmlFor="thumbnailPath">Thumbnail Path (Optional)</Label>
              <InputGroup>
                <InputWithButton
                  id="thumbnailPath"
                  type="text"
                  value={thumbnailPath}
                  onChange={handleThumbnailPathChange}
                  placeholder="Enter path to thumbnail image"
                  maxLength={500}
                />
                <BrowseButton
                  type="button"
                  variant="secondary"
                  onClick={handleBrowseThumbnail}
                  leftIcon={FaFolderOpen}
                >
                  Browse
                </BrowseButton>
              </InputGroup>
            </FormGroup>

            {errorMessage && errorMessage.length !== 0 && (
              <ErrorText>{errorMessage}</ErrorText>
            )}
          </ModalContentContainer>
          <ModalButtons>
            <Button
              type="button"
              variant="secondary"
              onClick={handleCancel}
              leftIcon={MdClose}
            >
              Cancel
            </Button>
            <Button type="button" variant="primary" onClick={handleAddGame}>
              Add Game
            </Button>
          </ModalButtons>
        </>
      )}
    </Modal>
  );
};
