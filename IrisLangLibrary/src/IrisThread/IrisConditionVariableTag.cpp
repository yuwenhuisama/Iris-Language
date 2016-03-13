#include "IrisThread/IrisConditionVariableTag.h"
#include "IrisThread/IrisThreadManager.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"


void IrisConditionVariableTag::Initialize(recursive_mutex * pMutex)
{
	m_pMutex = pMutex;
}

void IrisConditionVariableTag::NotifyOne()
{
	m_cvaConditionVariable.notify_one();
}

void IrisConditionVariableTag::NotifyAll()
{
	m_cvaConditionVariable.notify_all();
}

void IrisConditionVariableTag::Wait()
{
	IrisThreadManager::CurrentThreadManager()->SetThreadBlock(this_thread::get_id(), true);
	if (IrisThreadManager::CurrentThreadManager()->IsAllThreadBlocked()) {
		IrisGC::CurrentGC()->sm_cvGCStopTheWorldFinishedConditionVariable.notify_all();
	}
	m_cvaConditionVariable.wait(*m_pMutex);
}

IrisConditionVariableTag::IrisConditionVariableTag()
{
}


IrisConditionVariableTag::~IrisConditionVariableTag()
{
}
