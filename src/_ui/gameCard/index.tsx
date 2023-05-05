import 'react-modern-drawer/dist/index.css';

import fallback from '@assets/fallback.jpg';
import { useLibrary } from '@global/contexts/libraryProvider';
import {
  AiFillInfoCircle,
  AiFillStar,
  AiOutlineStar,
  FaPlay,
  HiDownload,
} from '@global/icons';
import { dialog, invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import * as React from 'react';
import Drawer from 'react-modern-drawer';
import styled from 'styled-components';

import Button from '../button';

const CardContainer = styled.div`
  flex: 0 0 auto;
  vertical-align: top;
  width: 15rem;
  height: 20rem;
  background-color: ${({ theme }) => theme.colors.secondary};
  border-radius: 0.5rem;
  margin: 0.5rem;
`;

const CardContent = styled.div`
  height: 90%;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  margin: 0.5rem;
`;

const Header = styled.div`
  position: relative;
  text-align: center;
`;

const Info = styled.p`
  font-weight: 700;
  color: ${({ theme }) => theme.colors.primary};
`;

const Thumbnail = styled.img<{ $isInfo?: boolean }>`
  border-radius: 0.5rem;
  width: 100%;
  height: ${({ $isInfo }) => ($isInfo ? '16.625rem' : '6.5rem')};
`;

const ButtonContainer = styled.div`
  display: flex;
  gap: 1rem;
`;

const StyledButton = styled(Button)<{ $isInfo?: boolean }>`
  background-color: ${({ $isInfo }) => ($isInfo ? 'blue' : 'green')};
  border-color: ${({ $isInfo }) => ($isInfo ? 'blue' : 'green')};
  color: white;

  &:hover,
  &:focus {
    background-color: ${({ $isInfo }) => ($isInfo ? 'darkblue' : 'darkgreen')};
    border-color: ${({ $isInfo }) => ($isInfo ? 'darkblue' : 'darkgreen')};
    color: white;
  }
`;

const FavouriteButton = styled(Button)`
  position: absolute;
  left: 85%;
`;

type GameCardProps = {
  id: string;
  platformId: string;
  executablePath: string;
  name: string;
  platform: string;
  thumbnailPath: string;
  isLibrary?: boolean;
};

const GameCard = ({
  id,
  platformId,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  executablePath,
  name,
  platform,
  thumbnailPath,
  isLibrary = false,
}: GameCardProps) => {
  const [drawerOpen, setDrawerOpen] = React.useState(false);
  const [isFavourite, setIsFavourite] = React.useState(false);
  const drawerRef = React.useRef<HTMLDivElement | null>(null);
  const { library } = useLibrary();

  const toggleDrawer = React.useCallback(() => {
    setDrawerOpen((prev) => !prev);
  }, []);

  const toggleFavourite = React.useCallback(() => {
    setIsFavourite((prev) => !prev);
  }, []);

  const imageSrc = React.useMemo<string>(() => {
    if (!thumbnailPath || thumbnailPath === ('' || 'temp')) {
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
      await dialog.message(`An error has occured: Could't download ${name}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [name, platformId, platform]);

  const hasGame = React.useMemo<boolean>(() => {
    return !!library.find((game) => game.id === id);
  }, [id, library]);

  // Detect click outside drawer to close it
  React.useEffect(() => {
    if (!drawerOpen) {
      return () => {};
    }

    const handleClickOutside = (event: any) => {
      if (drawerRef.current && !drawerRef.current.contains(event.target)) {
        toggleDrawer();
      }
    };
    document.addEventListener('mousedown', handleClickOutside);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [drawerRef, toggleDrawer, drawerOpen]);

  return (
    <CardContainer>
      <CardContent>
        <Header>
          {isLibrary && (
            <FavouriteButton
              type="button"
              variant="icon"
              onClick={toggleFavourite}
              title={
                isFavourite ? 'Remove from favourites' : 'Add to Favourites'
              }
            >
              {isFavourite ? (
                <AiFillStar size={24} color="yellow" transform="scale(1.2)" />
              ) : (
                <AiOutlineStar size={24} color="yellow" />
              )}
            </FavouriteButton>
          )}
          <Thumbnail
            alt="game-thumbnail"
            src={imageSrc}
            onError={handleImageError}
          />
          <Info>{name}</Info>
        </Header>
        <ButtonContainer>
          <StyledButton
            variant="primary"
            type="button"
            onClick={toggleDrawer}
            $isInfo
          >
            <AiFillInfoCircle size={24} />
          </StyledButton>
          <StyledButton
            variant="primary"
            type="button"
            onClick={hasGame ? handleLaunch : handleDownload}
          >
            {hasGame ? <FaPlay size={20} /> : <HiDownload size={24} />}
          </StyledButton>
        </ButtonContainer>
      </CardContent>
      <div ref={drawerRef}>
        <Drawer
          open={drawerOpen}
          direction="right"
          size={550}
          enableOverlay={false}
          style={drawerStyles}
        >
          <Thumbnail
            alt="game"
            src={imageSrc}
            onError={handleImageError}
            $isInfo
          />
          <Info>{name}</Info>
          <Info>Platform: {platform}</Info>
        </Drawer>
      </div>
    </CardContainer>
  );
};

export default GameCard;
