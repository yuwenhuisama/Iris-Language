#include "IrisThread/IrisThreadTag.h"
#include <sstream>
using namespace std;



IrisThreadTag::IrisThreadTag()
{
}

void IrisThreadTag::Initialize(thread * pThread)
{
	m_pThread = pThread;
}

void IrisThreadTag::Join()
{
	if(m_pThread->joinable()) {
		m_pThread->join();
	}
}

void IrisThreadTag::Detach()
{
	if (!m_pThread->joinable()) {
		m_pThread->detach();
	}
}

size_t IrisThreadTag::GetThreadId()
{
	size_t nThreadID;
	stringstream sstream;
	sstream << m_pThread->get_id();
	sstream >> nThreadID;

	return nThreadID;
}

thread * IrisThreadTag::GetThread()
{
	return m_pThread;
}

mutex & IrisThreadTag::GetMutex()
{
	return m_rmReadyMutex;
}

IrisThreadTag::~IrisThreadTag()
{
	if (m_pThread) {
		delete m_pThread;
	}
}
