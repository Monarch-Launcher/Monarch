import * as React from 'react';
import styled, { css } from 'styled-components';
import { AiOutlineSearch } from 'react-icons/ai';

const SearchContainer = styled.div`
  display: flex;
  max-height: 2rem;
`;

const StyledInput = styled.input`
  border-radius: 0.5rem 0 0 0.5rem;
  background-color: ${({ theme }) => theme.colors.black};
  border: none;
  padding: 0.5rem;
  &:focus {
    outline: none;
  }
`;

const SearchButton = styled.button<{ $disabled: boolean }>`
  border-radius: 0 40% 40% 0;
  transition: ease 0.2s;
  color: ${({ theme }) => theme.colors.primary};
  background-color: ${({ theme }) => theme.colors.secondary};
  border: none;
  display: flex;
  align-items: center;
  padding: 0.2rem 0.5rem;

  ${({ $disabled }) =>
    !$disabled &&
    css`
      &:hover {
        cursor: pointer;
        background-color: ${({ theme }) => theme.colors.primary};
        color: ${({ theme }) => theme.colors.secondary};
      }
    `}
`;

type SearchBarProps = {
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onSearchClick: () => void;
  buttonDisabled?: boolean;
};

const SearchBar = ({
  value,
  onChange,
  onSearchClick,
  buttonDisabled = false,
}: SearchBarProps) => {
  const handleKeyPressed = React.useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter') {
        onSearchClick();
      }
    },
    [onSearchClick],
  );

  return (
    <SearchContainer>
      <StyledInput
        value={value}
        onChange={onChange}
        onKeyDown={handleKeyPressed}
      />
      <SearchButton
        type="button"
        onClick={onSearchClick}
        disabled={buttonDisabled}
        $disabled={buttonDisabled}
      >
        <AiOutlineSearch size={24} />
      </SearchButton>
    </SearchContainer>
  );
};

export default SearchBar;
