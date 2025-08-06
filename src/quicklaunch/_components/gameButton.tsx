import Button from '@_ui/button';
import fallback from '@assets/fallback.jpg';
import { MonarchGame } from '@global/types';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import * as dialog from '@tauri-apps/plugin-dialog';
import * as React from 'react';
import styled from 'styled-components';

const ButtonWrapper = styled.div`
  &:focus-within {
    outline: 2px solid ${({ theme }) => theme.colors.primary};
    outline-offset: 2px;
    border-radius: 0.5rem;
  }
`;

const StyledButton = styled(Button)`
  font-size: 1.5rem;
  min-height: 3.75rem;
  display: flex;
  gap: 1rem;
`;

const Title = styled.p`
  margin: 1;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
`;

const Thumbnail = styled.img`
  height: 3rem;
  width: 6rem;
  border-radius: 0.5rem;
`;

type Props = {
  game: MonarchGame;
  isFocused?: boolean;
  onFocus?: () => void;
};

const GameButton = ({ game, isFocused = false, onFocus }: Props) => {
  const wrapperRef = React.useRef<HTMLDivElement>(null);

  const handleLaunch = React.useCallback(async () => {
    try {
      await invoke('launch_game', { game });
      // TODO: Close window instance
    } catch (err) {
      await dialog.message(`An error has occured: ${err}`, {
        title: 'Error',
        kind: 'error',
      });
    }
  }, [game]);

  const handleKeyDown = React.useCallback((event: React.KeyboardEvent) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      handleLaunch();
    }
  }, [handleLaunch]);

  // Auto-focus when isFocused prop changes to true
  React.useEffect(() => {
    if (isFocused && wrapperRef.current) {
      wrapperRef.current.focus();
    }
  }, [isFocused]);

  const imageSrc = React.useMemo<string>(() => {
    if (!game.thumbnail_path || game.thumbnail_path === '' || game.thumbnail_path === 'temp') {
      return fallback;
    }

    return convertFileSrc(game.thumbnail_path);
  }, [game.thumbnail_path]);

  return (
    <ButtonWrapper
      ref={wrapperRef}
      onKeyDown={handleKeyDown}
      onFocus={onFocus}
      tabIndex={isFocused ? 0 : -1} // Only make focused button tabbable
    >
      <StyledButton
        type="button"
        variant="secondary"
        onClick={handleLaunch}
        fullWidth
      >
        <Thumbnail src={imageSrc} alt="game thumnbail" />
        <Title>{game.name}</Title>
      </StyledButton>
    </ButtonWrapper>
  );
};

export default GameButton;
