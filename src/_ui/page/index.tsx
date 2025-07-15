import * as React from 'react';
import styled, { AnyStyledComponent } from 'styled-components';
import Logo from '@assets/logo.svg';

import NavigationMenu from './navigation-menu';

const PageContainer = styled.div`
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
`;

const ContentContainer = styled.div`
  padding: 0.5rem 1rem;
  width: 100%;
  flex: 1;
  padding-top: 120px;
  padding-bottom: 80px; // Add space for navbar
  overflow-y: auto;
  min-height: 0;
`;

const BottomMenuContainer = styled.div`
  width: 100%;
`;

// TODO: Proper svg setup for styled components
const StyledLogo = styled(Logo as AnyStyledComponent)`
  position: absolute;
  top: 1rem;
  left: 0rem;
  margin: 0;
  width: auto;
  height: 80px;
  max-width: none;
  max-height: none;
  .st0 {
    fill: ${({ theme }) => theme.colors.primary};
  }
`;

type PageProps = {
  children: React.ReactNode;
  hideMenu?: boolean;
};

const Page = ({ children, hideMenu = false }: PageProps) => {
  return (
    <PageContainer>
      <ContentContainer>
        <StyledLogo />
        {children}
      </ContentContainer>

      {!hideMenu && (
        <BottomMenuContainer>
          <NavigationMenu />
        </BottomMenuContainer>
      )}
    </PageContainer>
  );
};

export default Page;
