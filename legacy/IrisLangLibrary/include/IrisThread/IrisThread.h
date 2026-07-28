#ifndef _H_IRISTHREAD_
#define _H_IRISTHREAD_

#include "IrisDevHeader.h"
#include "IrisInterpreter.h"
#include "IrisThread/IrisThreadTag.h"
#include "IrisThread/IrisThreadManager.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"
#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"

class IrisThread : public IIrisClass
{
public:
	static void ThreadBlockRun(IrisClosureBlock* pBlock, IrisThreadTag* pNativeThreadObject, IrisObject* pThreadObject, IIrisThreadInfo* pThreadInfo) {
		unique_lock<mutex> ulLock(pNativeThreadObject->GetMutex());
		pThreadObject->SetPermanent(true);
		thread::id nId = this_thread::get_id();
		IrisThreadManager::CurrentThreadManager()->CurrentThreadManager()->AddNewThread(nId, pNativeThreadObject->GetThread());

		auto pNewThreadInfo = new IrisThreadInfo();

		IrisThreadManager::CurrentThreadManager()->CurrentThreadManager()->AddNewThreadInfo(nId, pNewThreadInfo);
		IrisGC::CurrentGC()->AddThreadGCData(nId, new IrisGC::ThreadGCData());
		static_cast<IrisContextEnvironment*>(pBlock->GetCurrentContextEnvironment())->m_bIsThreadMainContext = true;

		pBlock->Excute(nullptr, pNewThreadInfo);

		// Ignore Irregular having happened in thread.
		if (IrisInterpreter::CurrentInterpreter()->IrregularHappened(static_cast<IrisThreadInfo*>(pThreadInfo))) {
			IrisInterpreter::CurrentInterpreter()->UnregistIrregular(static_cast<IrisThreadInfo*>(pThreadInfo));
		}

		static_cast<IrisContextEnvironment*>(pBlock->GetCurrentContextEnvironment())->m_bIsThreadMainContext = false;
		IrisGC::CurrentGC()->TransferCurrentGCDataToMainThread();
		IrisGC::CurrentGC()->RemoveThreadGCData(nId);
		IrisThreadManager::CurrentThreadManager()->TransferCurrentThreadHeapToMainThread();
		IrisThreadManager::CurrentThreadManager()->DeleteThreadInfo(nId);
		IrisThreadManager::CurrentThreadManager()->DeleteThread(nId);
		IrisContextEnvironment* pTmpEnv = static_cast<IrisContextEnvironment*>(pBlock->GetCurrentContextEnvironment());
		while (pTmpEnv) {
			--pTmpEnv->m_nReferenced;
			pTmpEnv = pTmpEnv->m_pUpperContextEnvironment;
		}
		//delete pBlock;
		pThreadObject->SetPermanent(false);

		if (IrisThreadManager::CurrentThreadManager()->IsAllThreadBlocked()) {
			IrisGC::CurrentGC()->CurrentGC()->sm_cvGCStopTheWorldFinishedConditionVariable.notify_all();
		}
	}

public:
	static IrisValue InitializeFunction(const IrisValue& ivObj, IIrisValues* pParameters, IIrisValues* pVariableParameters, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
		IrisContextEnvironment* pTmpContextEnvironment = static_cast<IrisContextEnvironment*>(pContextEnvironment);
		if (!pTmpContextEnvironment->m_pClosureBlock) {
			IrisDevUtil::GroanIrregularWithString("A Thread Object needs a block to run.", pThreadInfo);
			return IrisDevUtil::Nil();
		}
		
		auto pThreadTag = IrisDevUtil::GetNativePointer<IrisThreadTag*>(ivObj);
		static_cast<IrisContextEnvironment*>(static_cast<IrisClosureBlock*>(pTmpContextEnvironment->GetClosureBlock())->GetCurrentContextEnvironment())->m_bIsThreadMainContext = true;
		unique_lock<mutex> ulLock(pThreadTag->GetMutex());
		auto pThread = new thread(ThreadBlockRun, static_cast<IrisClosureBlock*>(static_cast<IrisContextEnvironment*>(pContextEnvironment)->GetClosureBlock()), pThreadTag, static_cast<IrisObject*>(ivObj.GetIrisObject()), pThreadInfo);
		pTmpContextEnvironment->m_pClosureBlock = nullptr;
		pThreadTag->Initialize(pThread);
		return ivObj;
	}

	static IrisValue Join(const IrisValue& ivObj, IIrisValues* pParameters, IIrisValues* pVariableParameters, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
		auto pThreadTag = IrisDevUtil::GetNativePointer<IrisThreadTag*>(ivObj);
		IrisThreadManager::CurrentThreadManager()->SetThreadBlock(this_thread::get_id(), true);
		if (IrisThreadManager::CurrentThreadManager()->IsAllThreadBlocked()) {
			IrisGC::CurrentGC()->sm_cvGCStopTheWorldFinishedConditionVariable.notify_all();
		}
		pThreadTag->Join();
		IrisThreadManager::CurrentThreadManager()->SetThreadBlock(this_thread::get_id(), false);
		return IrisDevUtil::Nil();
	}

	static IrisValue Detach(const IrisValue& ivObj, IIrisValues* pParameters, IIrisValues* pVariableParameters, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
		auto pThreadTag = IrisDevUtil::GetNativePointer<IrisThreadTag*>(ivObj);
		pThreadTag->Detach();
		return IrisDevUtil::Nil();
	}

	static IrisValue GetID(const IrisValue& ivObj, IIrisValues* pParameters, IIrisValues* pVariableParameters, IIrisContextEnvironment* pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
		auto pThreadTag = IrisDevUtil::GetNativePointer<IrisThreadTag*>(ivObj);
		return IrisDevUtil::CreateInt(pThreadTag->GetThreadId());
	}

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Thread";
	}

	IIrisClass* NativeSuperClassDefine() const {
		return IrisDevUtil::GetClass("Object");
	}

	int GetTrustteeSize(void* pNativePointer) {
		return sizeof(IrisThreadTag);
	}

	void* NativeAlloc() {
		return new IrisThreadTag();
	}

	void NativeFree(void* pNativePointer) {
		delete static_cast<IrisThreadTag*>(pNativePointer);
	}

	void NativeClassDefine() {
		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "join", Join, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "detach", Detach, 0, false);
		IrisDevUtil::AddGetter(this, "@id", GetID);
	}

	IrisThread();
	~IrisThread();
};

#endif