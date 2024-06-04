import * as React from 'react';
import styled from 'styled-components';

const PageContainer = styled.div`
  width: 100%;
  height: 100%;
  display: flex;
  overflow-y: hidden;
`;

const ContentContainer = styled.div`
  padding: 0.5rem 1rem;
  width: 100%;
`;

const Title = styled.h1``;

type PageProps = {
  children: React.ReactNode;
  title?: string;
};

const Page = ({ children, hideMenu = false, title }: PageProps) => {
  return (
    <PageContainer>
      <ContentContainer>
        {title !== undefined && <Title>{title}</Title>}
        {children}
      </ContentContainer>
    </PageContainer>
  );
};

export default Page;
