import * as React from 'react';
import styled from 'styled-components';
import Page from '../../common/page';
import SearchBar from '../../common/searchBar';

const LibraryContainer = styled.div`
  width: 85%;
  height: calc(100% - 7rem);
  overflow-y: auto;
  border-radius: 0.5rem;
`;

const Library = () => {
  return (
    <Page title="Library">
      <SearchBar value="" onChange={() => {}} onSearchClick={() => {}} />
      <LibraryContainer>content here</LibraryContainer>
    </Page>
  );
};

export default Library;
