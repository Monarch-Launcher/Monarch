import Button from '@_ui/button';
import Error from '@_ui/error';
import GameCard from '@_ui/gameCard';
import Page from '@_ui/page';
import SearchBar from '@_ui/searchBar';
import Spinner from '@_ui/spinner';
import { useCollections } from '@global/contexts/collectionsProvider';
import { useLibrary } from '@global/contexts/libraryProvider';
import { FaFolderOpen, FaFolderPlus, FiRefreshCcw } from '@global/icons';
import type { MonarchGame } from '@global/types';
import { useDisclosure } from '@mantine/hooks';
import { invoke } from '@tauri-apps/api/core';
import * as dialog from '@tauri-apps/plugin-dialog';
import * as React from 'react';
import { flushSync } from 'react-dom';
import styled, { css } from 'styled-components';

import AddGameModal from './addGameManually/modal';
import Modal from './createCollection/modal';
import Collection from './showCollection/collection';

const LibraryContainer = styled.div`
  width: 100%;
  height: calc(100% - 7rem);
  overflow-y: auto;
  border-radius: 0.5rem;
  margin: 1rem 0;
  padding: 0 1rem; /* add horizontal gutters */
`;

const StyledRefreshIcon = styled(FiRefreshCcw)<{ $loading: boolean }>`
  ${({ $loading }) =>
    $loading &&
    css`
      animation: spin-animation 1.2s infinite;
    `}

  @keyframes spin-animation {
    0% {
      transform: rotate(359deg);
    }
    100% {
      transform: rotate(0);
    }
  }
`;

const GameContainer = styled.div`
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-start;
`;

const LibraryLayout = styled.div`
  display: flex;
  flex-direction: row;
  height: 100%;
`;

const Sidebar = styled.div`
  display: flex;
  flex-direction: column;
  align-items: stretch;
  min-width: 220px;
  max-width: 260px;
  padding: 1.5rem 1rem 1rem 0.5rem;
  gap: 1.5rem;
`;

const UmuNoticeBar = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem 1.5rem 0.75rem 1rem;
  margin: 0 1rem 1rem 1rem;
  border-radius: 0.5rem;
  background: rgba(255, 193, 7, 0.1);
  border: 1px solid rgba(255, 193, 7, 0.35);
`;

const UmuNoticeText = styled.p`
  margin: 0;
  font-size: 0.95rem;
`;

const StackedButton = styled(Button)`
  height: 3.5rem;
  font-size: 1rem;
  justify-content: flex-start;
  padding-left: 0.75rem;
  margin-bottom: 1rem;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
  svg {
    margin-right: 0.75rem;
  }
`;

const SidebarButtonGroup = styled.div`
  display: flex;
  flex-direction: column;
  gap: 1rem;
`;

const Library = () => {
  const [searchTerm, setSearchTerm] = React.useState('');
  const [dialogError, setDialogError] = React.useState(false);
  const [opened, { open, close }] = useDisclosure(false);
  const [addGameOpened, { open: openAddGame, close: closeAddGame }] =
    useDisclosure(false);
  const [selectedFilePath, setSelectedFilePath] = React.useState<
    string | undefined
  >();
  const { library, loading, error, refreshLibrary, results } = useLibrary();
  const { collections } = useCollections();

  // umu-launcher notification state (Linux only)
  const [showUmuNotice, setShowUmuNotice] = React.useState(false);
  const [umuChecking, setUmuChecking] = React.useState(true);
  const [umuInstalling, setUmuInstalling] = React.useState(false);

  React.useEffect(() => {
    // Only run under Linux
    const isLinux = navigator.userAgent.toLowerCase().includes('linux');
    if (!isLinux) {
      setUmuChecking(false);
      return;
    }

    let cancelled = false;
    const checkUmu = async () => {
      try {
        const installed = await invoke<boolean>('umu_is_installed');
        if (!cancelled) {
          setShowUmuNotice(!installed);
        }
      } catch (err) {
        // On error, be non-intrusive: hide the notice and optionally log
        if (!cancelled) setShowUmuNotice(false);
      } finally {
        if (!cancelled) setUmuChecking(false);
      }
    };
    checkUmu();
    // eslint-disable-next-line consistent-return
    return () => {
      cancelled = true;
    };
  }, []);

  const handleOpenDialog = React.useCallback(async () => {
    try {
      setDialogError(false);

      const selected = await dialog.open({
        multiple: false,
        title: 'Choose a game executable',
        directory: false,
        filters: [
          {
            name: 'Executables',
            extensions: ['exe', 'app', 'sh', 'bin', 'run', 'x86_64'],
          },
          {
            name: 'All Files',
            extensions: ['*'],
          },
        ],
      });

      if (selected) {
        setSelectedFilePath(selected as string);
        openAddGame();
      }
    } catch (err) {
      setDialogError(true);
    }
  }, [openAddGame]);

  const handleSearchTermChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchTerm(e.target.value);
    },
    [],
  );

  const filteredLibrary = React.useMemo<MonarchGame[]>(() => {
    // Contains all gameIds that are in a collection
    const gamesInCollection = collections.flatMap((col) => col.gameIds);
    // Contains games that are not in a collection
    const notInCollection = library.filter(
      (game) => !gamesInCollection.includes(game.id),
    );

    return notInCollection
      .filter((game) =>
        game.name
          .replace(/[.,/#!$%^&*;:{}=\-_`~()]/g, '')
          .toLowerCase()
          .match(searchTerm.toLowerCase()),
      )
      .sort((a, b) => a.name.localeCompare(b.name));
  }, [library, searchTerm, collections]);

  return (
    <Page>
      <LibraryLayout>
        <Sidebar>
          <SearchBar
            value={searchTerm}
            onChange={handleSearchTermChange}
            placeholder="Search Library"
            fullWidth
          />
          <SidebarButtonGroup>
            <StackedButton
              type="button"
              variant="primary"
              onClick={refreshLibrary}
              title={loading ? 'Loading...' : 'Refresh'}
              loading={loading}
              fullWidth
            >
              <StyledRefreshIcon $loading={loading} />
              Refresh Library
            </StackedButton>
            <StackedButton
              type="button"
              variant="primary"
              onClick={open}
              title="Add new collection"
              fullWidth
            >
              <FaFolderPlus />
              New Collection
            </StackedButton>
            <StackedButton
              type="button"
              variant="primary"
              onClick={handleOpenDialog}
              title="Add game executable"
              fullWidth
            >
              <FaFolderOpen />
              Add Game Manually
            </StackedButton>
          </SidebarButtonGroup>
          <Modal opened={opened} close={close} library={library} />
          <AddGameModal
            opened={addGameOpened}
            close={closeAddGame}
            selectedFilePath={selectedFilePath}
            onGameAdded={refreshLibrary}
          />
        </Sidebar>
        <LibraryContainer>
          {showUmuNotice && !umuChecking && (
            <UmuNoticeBar>
              <UmuNoticeText>
                UMU Launcher is not installed. Some features may require it.
              </UmuNoticeText>
              <Button
                type="button"
                variant="primary"
                loading={umuInstalling}
                onClick={async () => {
                  try {
                    // Flush state synchronously so the label updates immediately
                    flushSync(() => setUmuInstalling(true));
                    // Force a paint before starting installation (more reliable on WebKitGTK)
                    await new Promise<void>((resolve) =>
                      { requestAnimationFrame(() => requestAnimationFrame(() => resolve())) },
                    );
                    await invoke('install_umu');
                    setShowUmuNotice(false);
                  } catch (err) {
                    await dialog.message(`Failed to install umu-launcher: ${err}`, {
                      title: 'Error',
                    });
                  } finally {
                    setUmuInstalling(false);
                  }
                }}
              >
                {umuInstalling ? 'Downloading...' : 'Download umu-launcher'}
              </Button>
            </UmuNoticeBar>
          )}
          {collections.length !== 0 && (
            <>
              {collections.map((collection) => (
                <Collection key={collection.id} collection={collection} />
              ))}
            </>
          )}
          <GameContainer>
            {filteredLibrary.length === 0 && loading ? (
              <Spinner />
            ) : (
              filteredLibrary.map((game) => (
                <GameCard
                  key={game.id}
                  id={game.id}
                  executablePath={game.executable_path}
                  platform={game.platform}
                  name={game.name}
                  platformId={game.platform_id}
                  thumbnailPath={game.thumbnail_path}
                  storePage={game.store_page}
                  isLibrary
                />
              ))
            )}
          </GameContainer>

          {!loading && results?.empty && <p>{results.emptyMessage}</p>}
          {error && (
            <Error
              description="Couldn't load library"
              onRetry={refreshLibrary}
            />
          )}
          {dialogError && (
            <Error
              description="Couldn't open file explorer"
              onRetry={handleOpenDialog}
            />
          )}
        </LibraryContainer>
      </LibraryLayout>
    </Page>
  );
};

export default Library;
