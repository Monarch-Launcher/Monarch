import * as React from 'react';
import styled from 'styled-components';
import { AiOutlineSearch } from 'react-icons/ai';
import Button from '../button';

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

const SearchButton = styled(Button)`
  border-radius: 0 40% 40% 0;
  border: none;
  padding: 0.2rem 0.5rem;

  &:hover {
    border: none;
    background-color: ${({ theme }) => theme.colors.primary};
    color: ${({ theme }) => theme.colors.secondary};
  }

  &:focus {
    border: none;
  }
`;

type SearchBarProps = {
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onSearchClick: () => void;
  buttonDisabled?: boolean;
  placeholder?: string;
  loading?: boolean;
};

const SearchBar = ({
  value,
  onChange,
  onSearchClick,
  buttonDisabled = false,
  placeholder,
  loading = false,
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
        placeholder={placeholder}
      />
      <SearchButton
        type="button"
        variant="secondary"
        onClick={onSearchClick}
        disabled={buttonDisabled}
        loading={loading}
      >
        <AiOutlineSearch size={24} />
      </SearchButton>
    </SearchContainer>
  );
};

export default SearchBar;
