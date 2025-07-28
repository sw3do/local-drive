import { NextRequest, NextResponse } from 'next/server';

export default function middleware(request: NextRequest) {
  const pathname = request.nextUrl.pathname;
  const token = request.cookies.get('auth-token')?.value;
  
  const publicPaths = ['/login'];
  const isPublicPath = publicPaths.some(path => 
    pathname === path
  );
  
  if (isPublicPath && token) {
    return NextResponse.redirect(new URL('/', request.url));
  }
  
  if (!isPublicPath && !pathname.includes('/api')) {
    if (!token) {
      return NextResponse.redirect(new URL('/login', request.url));
    }
  }
  
  return NextResponse.next();
}

export const config = {
  matcher: ['/((?!api|_next|_vercel|.*\..*).*)'],
};