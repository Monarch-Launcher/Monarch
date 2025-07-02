import 'react-modern-drawer/dist/index.css';

import fallback from '@assets/fallback.jpg';
import { useLibrary } from '@global/contexts/libraryProvider';
import {
  FaPlay,
  FaRegEdit,
  FaSteam,
  HiDownload,
  PiButterflyBold,
  SiEpicgames,
} from '@global/icons';
import { dialog, invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import * as React from 'react';
import { FiInfo } from 'react-icons/fi';
import styled, { keyframes } from 'styled-components';

import Button from '../button';

const CardWrapper = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 24rem;
  margin: 0.5rem;
`;

const CardContainer = styled.div`
  position: relative;
  width: 100%;
  height: 20rem;
  background-color: ${({ theme }) => theme.colors.secondary};
  border-radius: 0.5rem;
  overflow: hidden; /* Ensure the image doesn't overflow the card */

  &:hover button {
    opacity: 1;
  }
`;

const Thumbnail = styled.img<{ $isInfo?: boolean }>`
  width: 100%;
  height: 100%;
  object-fit: cover; /* Ensures the image covers the entire area without distortion */
  position: absolute; /* Position the image absolutely to fill the entire card */
  top: 0;
  left: 0;
  z-index: 0; /* Place it below the text and buttons */
`;

const StyledButton = styled(Button)<{ $isInfo?: boolean }>`
  background-color: ${({ $isInfo }) => ($isInfo ? 'grey' : 'orange')};
  border-color: ${({ $isInfo }) => ($isInfo ? 'grey' : 'orange')};
  color: white;
  z-index: 4; /* Ensure buttons are on top */

  &:hover,
  &:focus {
    background-color: ${({ $isInfo }) => ($isInfo ? 'darkgrey' : 'darkorange')};
    border-color: ${({ $isInfo }) => ($isInfo ? 'darkgrey' : 'darkorange')};
    color: white;
  }
`;

const HoverButtonWrapper = styled.div`
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  opacity: 0;
  transition: opacity 0.3s ease;

  ${CardContainer}:hover & {
    opacity: 1;
  }
`;

const Info = styled.p`
  font-weight: 700;
  color: ${({ theme }) => theme.colors.primary};
  margin-top: 0.5rem;
  text-align: center;
`;

const OptionsButton = styled.button`
  position: absolute;
  bottom: 0.5rem;
  left: 0.5rem;
  background: rgba(30, 30, 30, 0.85);
  border: 1px solid ${({ theme }) => theme.colors.primary};
  border-radius: 50%;
  padding: 0.3rem;
  min-width: 2.5rem;
  min-height: 2.5rem;
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  color: ${({ theme }) => theme.colors.primary};
  cursor: pointer;
  z-index: 5;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
  opacity: 0;
  pointer-events: none;
  transition: background 0.2s, opacity 0.2s, border 0.2s, color 0.2s;
  svg {
    color: ${({ theme }) => theme.colors.primary};
    transition: color 0.2s;
  }
  ${CardContainer}:hover & {
    opacity: 1;
    pointer-events: auto;
  }
  &:hover {
    background: rgba(30, 30, 30, 1);
    color: #fff;
    border: 1px solid ${({ theme }) => theme.colors.primary};
    svg {
      color: #fff;
    }
  }
`;

const DropdownMenu = styled.div`
  position: absolute;
  top: 2.5rem;
  right: 0.5rem;
  background: ${({ theme }) => theme.colors.secondary};
  border: 1px solid ${({ theme }) => theme.colors.primary};
  border-radius: 0.5rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  z-index: 10;
  min-width: 8rem;
  padding: 0.5rem 0;
  display: flex;
  flex-direction: column;
  pointer-events: auto;
`;

const DropdownItem = styled.button`
  background: none;
  border: none;
  color: ${({ theme }) => theme.colors.primary};
  padding: 0.5rem 1rem;
  text-align: left;
  width: 100%;
  cursor: pointer;
  font-size: 1rem;
  &:hover {
    background: ${({ theme }) => theme.colors.button.primary.hoverBackground};
    color: ${({ theme }) => theme.colors.button.primary.hoverText};
  }
`;

const DrawerOverlay = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
  align-items: stretch;
`;

const slideIn = keyframes`
  from {
    transform: translateX(100%);
    opacity: 0.5;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
`;

const Drawer = styled.div`
  position: relative;
  width: 600px;
  max-width: 98vw;
  height: 90vh;
  margin: auto 0;
  box-shadow: -2px 0 16px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  animation: ${slideIn} 0.35s cubic-bezier(0.4, 0, 0.2, 1);
`;

const DrawerCloseButton = styled(Button)`
  position: absolute;
  top: 1rem;
  right: 1rem;
  z-index: 10;
`;

const DrawerTitle = styled.h2`
  color: ${({ theme }) => theme.colors.primary};
  margin-bottom: 1rem;
`;

const DrawerButtonRow = styled.div`
  display: flex;
  gap: 1rem;
  margin-top: 2rem;
`;

const StoreIconButton = styled(Button)`
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem 1rem;
  font-size: 1.2rem;
`;

const DetailsButton = styled(Button)`
  position: absolute;
  bottom: 0.5rem;
  right: 0.5rem;
  z-index: 6;
  background: rgba(30, 30, 30, 0.85);
  border: 1px solid ${({ theme }) => theme.colors.primary};
  border-radius: 50%;
  padding: 0.3rem;
  min-width: 2.5rem;
  min-height: 2.5rem;
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  color: ${({ theme }) => theme.colors.primary};
  opacity: 0;
  pointer-events: none;
  transition: background 0.2s, opacity 0.2s, color 0.2s;
  ${CardContainer}:hover & {
    opacity: 1;
    pointer-events: auto;
  }
  &:hover {
    background: rgba(30, 30, 30, 1);
    color: #fff;
  }
`;

type GameCardProps = {
  id: string;
  platformId: string;
  executablePath: string;
  name: string;
  platform: string;
  thumbnailPath: string;
  storePage: string;
  isLibrary?: boolean;
  cardWidth?: string;
};

const GameCard = ({
  id,
  platformId,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  executablePath,
  name,
  platform,
  thumbnailPath,
  storePage,
  isLibrary = false,
  cardWidth = '15rem',
}: GameCardProps) => {
  const [drawerOpen, setDrawerOpen] = React.useState(false);
  const drawerRef = React.useRef<HTMLDivElement | null>(null);
  const { library } = useLibrary();
  const [optionsOpen, setOptionsOpen] = React.useState(false);
  const optionsRef = React.useRef<HTMLButtonElement | null>(null);

  const toggleDrawer = React.useCallback(() => {
    setDrawerOpen((prev) => !prev);
  }, []);

  const imageSrc = React.useMemo<string>(() => {
    if (!thumbnailPath || thumbnailPath === 'temp') {
      return fallback;
    }

    return convertFileSrc(thumbnailPath);
  }, [thumbnailPath]);

  const handleImageError = React.useCallback(
    (e: React.SyntheticEvent<HTMLImageElement, Event>) => {
      e.currentTarget.src = fallback;
    },
    [],
  );

  const drawerStyles = React.useMemo<React.CSSProperties>(() => {
    return {
      backgroundColor: 'rgba(15, 15, 15, 0.95)',
      borderRadius: '0.5rem',
    };
  }, []);

  const handleLaunch = React.useCallback(async () => {
    try {
      await invoke('launch_game', { name, platformId, platform });
    } catch (err) {
      await dialog.message(`An error has occured: Could't launch ${name}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [name, platformId, platform]);

  const handleDownload = React.useCallback(async () => {
    try {
      await invoke('download_game', {
        name,
        platformId,
        platform,
      });
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [name, platformId, platform]);

  const handleUpdate = React.useCallback(async () => {
    try {
      await invoke('update_game', {
        name,
        platformId,
        platform,
      });
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [name, platformId, platform]);

  const handleUninstallGame = React.useCallback(async () => {
    try {
      await invoke('remove_game', {
        name,
        platformId,
        platform,
      });
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [name, platformId, platform]);

  const hasGame = React.useMemo<boolean>(() => {
    return !!library.find((game) => game.id === id);
  }, [id, library]);

  const getStoreIcon = React.useMemo(() => {
    switch (platform) {
      case 'steam':
        return FaSteam;
      case 'epic':
        return SiEpicgames;
      default:
        return PiButterflyBold;
    }
  }, [platform]);

  const openStorePage = React.useCallback(async () => {
    try {
      await invoke('open_store', {
        url: storePage,
      });
    } catch (err) {
      await dialog.message(
        `An error has occured: Could not open store page ${storePage}`,
        {
          title: 'Error',
          type: 'error',
        },
      );
    }
  }, [storePage]);

  React.useEffect(() => {
    if (!optionsOpen) return;
    const handleClick = (e: MouseEvent) => {
      if (
        optionsRef.current &&
        !optionsRef.current.contains(e.target as Node)
      ) {
        setOptionsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    // eslint-disable-next-line consistent-return
    return () => document.removeEventListener('mousedown', handleClick);
  }, [optionsOpen]);

  return (
    <CardWrapper style={{ width: cardWidth }}>
      <CardContainer>
        <Thumbnail
          alt="game-thumbnail"
          src={imageSrc}
          onError={handleImageError}
        />
        {/* Options Button (dropdown) at bottom left */}
        <OptionsButton
          ref={optionsRef}
          onClick={() => setOptionsOpen((v) => !v)}
          title="Game options"
        >
          <FaRegEdit size={20} />
        </OptionsButton>
        {optionsOpen && (
          <DropdownMenu
            style={{ left: 0, bottom: '3rem', right: 'auto', top: 'auto' }}
          >
            <DropdownItem
              onMouseDown={async (e) => {
                e.stopPropagation();
                await openStorePage();
                setOptionsOpen(false);
              }}
            >
              Open Store Page
            </DropdownItem>
            {isLibrary && (
              <>
                <DropdownItem
                  onMouseDown={async (e) => {
                    e.stopPropagation();
                    await handleUpdate();
                    setOptionsOpen(false);
                  }}
                >
                  Update
                </DropdownItem>
                <DropdownItem
                  onMouseDown={async (e) => {
                    e.stopPropagation();
                    await handleUninstallGame();
                    setOptionsOpen(false);
                  }}
                >
                  Uninstall
                </DropdownItem>
              </>
            )}
          </DropdownMenu>
        )}
        {/* Info Icon Button to open drawer */}
        <DetailsButton
          type="button"
          variant="icon"
          onClick={toggleDrawer}
          title="Show details"
        >
          <FiInfo size={20} />
        </DetailsButton>
        <HoverButtonWrapper>
          <StyledButton
            variant="primary"
            type="button"
            onClick={hasGame ? handleLaunch : handleDownload}
          >
            {hasGame ? <FaPlay size={20} /> : <HiDownload size={24} />}
          </StyledButton>
        </HoverButtonWrapper>
      </CardContainer>
      <Info>{name}</Info>
      {/* Custom Drawer for game details */}
      {drawerOpen && (
        <DrawerOverlay onClick={toggleDrawer}>
          <Drawer
            style={drawerStyles}
            onClick={(e) => e.stopPropagation()}
            ref={drawerRef}
          >
            <DrawerCloseButton
              type="button"
              variant="icon"
              onClick={toggleDrawer}
              title="Close"
            >
              ×
            </DrawerCloseButton>
            <DrawerTitle>{name}</DrawerTitle>
            <Thumbnail
              alt="game-thumbnail"
              src={imageSrc}
              onError={handleImageError}
              style={{
                position: 'static',
                width: '80%',
                height: 'auto',
                borderRadius: '0.5rem',
                marginBottom: '1rem',
              }}
            />
            <p style={{ color: '#aaa', marginBottom: '1rem' }}>
              Platform: {platform}
            </p>
            <DrawerButtonRow>
              <StyledButton
                variant="primary"
                type="button"
                onClick={hasGame ? handleLaunch : handleDownload}
              >
                {hasGame ? <FaPlay size={20} /> : <HiDownload size={24} />}
                {hasGame ? 'Launch' : 'Download'}
              </StyledButton>
              {isLibrary && (
                <StyledButton
                  variant="secondary"
                  type="button"
                  onClick={handleUpdate}
                >
                  Update
                </StyledButton>
              )}
              {isLibrary && (
                <StyledButton
                  variant="danger"
                  type="button"
                  onClick={handleUninstallGame}
                >
                  Uninstall
                </StyledButton>
              )}
              <StoreIconButton
                variant="secondary"
                type="button"
                onClick={openStorePage}
                title="Open Store Page"
              >
                {React.createElement(getStoreIcon, {
                  size: 24,
                  style: { marginRight: 8 },
                })}
                Store
              </StoreIconButton>
            </DrawerButtonRow>
          </Drawer>
        </DrawerOverlay>
      )}
    </CardWrapper>
  );
};

export default GameCard;
