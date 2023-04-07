import * as React from 'react';
import styled from 'styled-components';
import SideMenu from './sidemenu';

const PageContainer = styled.div`
  width: 100%;
  height: 100%;
  display: flex;
`;

const ContentContainer = styled.div`
  padding: 0.5rem;
`;

type PageProps = {
  children: React.ReactNode;
  hideMenu?: boolean;
};

const Page = ({ children, hideMenu = false }: PageProps) => {
  return (
    <PageContainer>
      {!hideMenu && <SideMenu />}
      <ContentContainer>{children}</ContentContainer>
    </PageContainer>
  );
};

export default Page;
