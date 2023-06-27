
import { Inter } from 'next/font/google'
import Head from 'next/head'
import AppList from '@/components/AppsList'
import React, { useState, useEffect } from 'react';
import socketio from 'socket.io-client';

export default function Home() {
  const [apps, setApps] = useState([])
  const [socket, setSocket] = useState(null);
  const [currentApp, setCurrentApp] = useState(undefined);
  const [currentRequest, setCurrentRequest] = useState(undefined);

  useEffect(() => {
    const newSocket = socketio.connect('http://localhost:1337');
    setSocket(newSocket);
    return () => newSocket.disconnect();
  }, []);

  const handleMessageSubmit = event => {
    event.preventDefault();
    const message = event.target.message.value;
    console.log(`Sending message to server: ${message}`);
    socket.emit('message', message);
    event.target.reset();
  };

  useEffect(() => {
    if (!socket) return;
    socket.on('message', message => {
      console.log(`Received message from server:`, message);
      if(message.type === 'apps') {
        setApps(message.payload)        
        if(currentApp !== undefined) {          
          message.payload.forEach(sp => {
            if(sp.id === currentApp.id) {
              setCurrentApp(sp)              
            }
          })          
        }
      } else if (message.type === 'add_request') {
        setCurrentRequest(message.payload)
      } else if (message.type === 'update_request') {
        setCurrentRequest(message.payload)
      }
    });
  }, [socket, currentApp]);

  return (
    <>   
    <Head>
        <title>Apps - Seaplane</title>
    </Head>
      <div className="min-h-full">                                   
        <AppList apps={apps} currentApp={currentApp} setCurrentApp={setCurrentApp} currentRequest={currentRequest}/>
      </div>
    </>
  )
}
