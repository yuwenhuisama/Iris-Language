#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisCompiler.h"
#include "IrisInterpreter/IrisStructure/IrisContextEnvironment.h"

#include "IrisInterpreter/IrisNativeClasses/IrisInteger.h"
#include "IrisInterpreter/IrisNativeClasses/IrisFloat.h"
#include "IrisInterpreter/IrisNativeClasses/IrisString.h"
#include "IrisInterpreter/IrisNativeClasses/IrisUniqueString.h"
#include "IrisInterpreter/IrisNativeClasses/IrisClassBase.h"
#include "IrisInterpreter/IrisNativeClasses/IrisObjectBase.h"
#include "IrisInterpreter/IrisNativeClasses/IrisClassBase.h"
#include "IrisInterpreter/IrisNativeClasses/IrisModuleBase.h" 
#include "IrisInterpreter/IrisNativeClasses/IrisFalseClass.h"
#include "IrisInterpreter/IrisNativeClasses/IrisTrueClass.h"
#include "IrisInterpreter/IrisNativeClasses/IrisNilClass.h"
#include "IrisInterpreter/IrisNativeClasses/IrisIrregular.h"
#include "IrisInterpreter/IrisNativeClasses/IrisArray.h"
#include "IrisInterpreter/IrisNativeClasses/IrisArrayIterator.h"
#include "IrisInterpreter/IrisNativeClasses/IrisHash.h"
#include "IrisInterpreter/IrisNativeClasses/IrisHashIterator.h"
#include "IrisInterpreter/IrisNativeClasses/IrisRange.h"
#include "IrisInterpreter/IrisNativeClasses/IrisRangeIterator.h"
#include "IrisInterpreter/IrisNativeClasses/IrisDummyClass.h"

#include "IrisInterpreter/IrisNativeClasses/IrisMethodBase.h"
#include "IrisInterpreter/IrisNativeClasses/IrisInterfaceBase.h"
#include "IrisInterpreter/IrisNativeModules/IrisGCModule.h"
#include "IrisInterpreter/IrisNativeModules/IrisKernel.h"
#include "IrisInterpreter/IrisNativeClasses/IrisClosureBlockBase.h"
#include "IrisInterpreter/IrisNativeModules/IrisDummyModule.h"

#include "IrisInterpreter/IrisNativeInterfaces/IrisDummyInterface.h"

#include "IrisFatalErrorHandler.h"

#include "IrisComponents/IrisComponentsDefines.h"

#include "IrisDevelopUtil.h"
#include "IrisUnil/IrisMemoryPool/IrisAbstractMemoryPool.h"

#include "IrisThread/IrisThreadManager.h"
#include "IrisThread/IrisThread.h"
#include "IrisThread/IrisMutex.h"
#include "IrisThread/IrisConditionVariable.h"

#include "IrisVirtualCodeNumber.h"

//#include "IrisExtentionUtil.h"

IrisInterpreter* IrisInterpreter::s_pInstance = nullptr;


bool IrisInterpreter::Initialize() {

	IrisThreadManager::CurrentThreadManager()->Initialize();
	IrisGC::CurrentGC()->Initialize();
	IrisAbstractMemoryPool::Initialize();

	// Maybe a little mass....
 	RegistClass("Class", new IrisClassBase());
	INNER_CLASS_SET_POINTER(Class, GetIrisClass("Class")->GetExternClass());
	//GetIrisClass("Class")->ResetNativeObject();
	bool bResult = false;
	//((IrisValue&)GetConstance("Class", bResult)).SetIrisObject(GetIrisClass("Class")->GetClassObject());
	RegistClass("Module", new IrisModuleBase());
	INNER_CLASS_SET_POINTER(Module, GetIrisClass("Module")->GetExternClass());
	RegistClass("Interface", new IrisInterfaceBase());
	INNER_CLASS_SET_POINTER(Interface, GetIrisClass("Interface")->GetExternClass());
	RegistModule("Kernel", new IrisKernel());
	RegistClass("Object", new IrisObjectBase());
	INNER_CLASS_SET_POINTER(Object, GetIrisClass("Object")->GetExternClass());

	GetIrisClass("Object")->m_eClassType = IrisClass::ClassType::ObjectClass;
	GetIrisClass("Class")->SetSuperClass(GetIrisClass("Object"));
	GetIrisClass("Class")->m_eClassType = IrisClass::ClassType::ClassClass;
	GetIrisClass("Module")->SetSuperClass(GetIrisClass("Object"));
	GetIrisClass("Module")->m_eClassType = IrisClass::ClassType::ModuleClass;
	GetIrisClass("Interface")->SetSuperClass(GetIrisClass("Object"));
	GetIrisClass("Interface")->m_eClassType = IrisClass::ClassType::InterfaceClass;

	RegistClass("Method", new IrisMethodBase());

	GetIrisClass("Class")->ResetAllMethodsObject();
	GetIrisClass("Object")->ResetAllMethodsObject();
	GetIrisClass("Module")->ResetAllMethodsObject();
	GetIrisClass("Interface")->ResetAllMethodsObject();
	GetIrisModule("Kernel")->ResetAllMethodsObject();
	GetIrisClass("Method")->ResetAllMethodsObject();

	RegistClass("String", new IrisString());
	RegistClass("UniqueString", new IrisUniqueString());
	RegistClass("Integer", new IrisInteger());
	RegistClass("Float", new IrisFloat());

	RegistClass("TrueClass", new IrisTrueClass());
	RegistClass("FalseClass", new IrisFalseClass());
	RegistClass("NilClass", new IrisNilClass());

	RegistClass("Irregular", new IrisIrregular());
	RegistClass("Array", new IrisArray());
	RegistClass("ArrayIterator", new IrisArrayIterator());
	RegistClass("Hash", new IrisHash());
	RegistClass("HashIterator", new IrisHashIterator());
	RegistClass("Range", new IrisRange());
	RegistClass("RangeIterator", new IrisRangeIterator());

	RegistModule("GC", new IrisGCModule());

	RegistClass("Block", new IrisClosureBlockBase());
	RegistClass("Thread", new IrisThread());
	RegistClass("Mutex", new IrisMutex());
	RegistClass("ConditionVariable", new IrisConditionVariable());

	auto pInfo = IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo();
	// True/False/Nil
	m_ivTrue = GetIrisClass("TrueClass")->CreateInstance(nullptr, nullptr, pInfo);
	m_ivFalse = GetIrisClass("FalseClass")->CreateInstance(nullptr, nullptr, pInfo);
	m_ivNil = GetIrisClass("NilClass")->CreateInstance(nullptr, nullptr, pInfo);

	INNER_CLASS_SET_POINTER(String, GetIrisClass("String")->GetExternClass());
	INNER_CLASS_SET_POINTER(UniqueString, GetIrisClass("UniqueString")->GetExternClass());
	INNER_CLASS_SET_POINTER(Integer, GetIrisClass("Integer")->GetExternClass());
	INNER_CLASS_SET_POINTER(Float, GetIrisClass("Float")->GetExternClass());
	INNER_CLASS_SET_POINTER(Array, GetIrisClass("Array")->GetExternClass());
	INNER_CLASS_SET_POINTER(Hash, GetIrisClass("Hash")->GetExternClass());
	INNER_CLASS_SET_POINTER(Range, GetIrisClass("Range")->GetExternClass());
	INNER_CLASS_SET_POINTER(Block, GetIrisClass("Block")->GetExternClass());

	return true;
}

bool IrisInterpreter::ShutDown() {
	IrisGC::CurrentGC()->ReleaseAllThreadData();
	IrisThreadManager::CurrentThreadManager()->DetachAllThread();

	for (auto& pObject : m_hpHeap.GetHeapSet()) {
		if(pObject->GetClass()->GetInternClass()->IsNormalClass()) {
			delete pObject;
		}
	}

	IrisAbstractMemoryPool::GetCurrentMemoryPool()->ReleaseAllMemoryPool();
	m_pCurrentCompiler->Release();

	typedef bool(*RelFunc)();

	for (auto hModule : m_emLoadedExtention) {
		RelFunc pfFunc = (RelFunc)GetProcAddress(hModule.second, "IR_Release");
		pfFunc();
		::FreeLibrary(hModule.second);
	}

	return true;
}

IrisInterpreter::IrisInterpreter()
{
}

IrisInterpreter * IrisInterpreter::CurrentInterpreter()
{
	if (!s_pInstance) {
		s_pInstance = new IrisInterpreter();
	}
	return s_pInstance;
}

void IrisInterpreter::SetCurrentInterpreter(IrisInterpreter * pInter)
{
	s_pInstance = pInter;
}

#if IR_USE_STL_STRING
int IrisInterpreter::_Split(const string& str, list<string>& ret_, string sep) {
#else
int IrisInterpreter::_Split(const string& str, list<IrisInternString>& ret_, string sep) {
#endif // IR_USE_STL_STRING
	if (str.empty())
	{
		return 0;
	}

	string tmp;
	string::size_type pos_begin = str.find_first_not_of(sep);
	string::size_type comma_pos = 0;

	while (pos_begin != string::npos)
	{
		comma_pos = str.find(sep, pos_begin);
		if (comma_pos != string::npos)
		{
			tmp = str.substr(pos_begin, comma_pos - pos_begin);
			pos_begin = comma_pos + sep.length();
		}
		else
		{
			tmp = str.substr(pos_begin);
			pos_begin = comma_pos;
		}

		if (!tmp.empty())
		{
			ret_.push_back(tmp);
			tmp.clear();
		}
	}
	return 0;
}

#if IR_USE_STL_STRING
IrisModule * IrisInterpreter::_GetLastModuleFromPath(const list<string>& lsPath)
#else
IrisModule * IrisInterpreter::_GetLastModuleFromPath(const list<IrisInternString>& lsPath)
#endif // IR_USE_STL_STRING
{
	bool bTmpResult = false;
	auto ivIter = lsPath.begin();
	auto& ivConst = GetConstance(*(ivIter++), bTmpResult);
	if (!bTmpResult) {
		return false;
	}
	if (!IrisDevUtil::CheckClassIsModule(ivConst)) {
		return nullptr;
	}

	IrisModule* pCurModule = IrisDevUtil::GetNativeModule(ivConst)->GetInternModule();
	for (; ivIter != lsPath.end(); ++ivIter) {
		auto& ivConst = pCurModule->GetCurrentModuleConstance(*ivIter, bTmpResult);
		if (!bTmpResult) {
			return nullptr;
		}
		if (!IrisDevUtil::CheckClassIsModule(ivConst)) {
			return nullptr;
		}
		pCurModule = IrisDevUtil::GetNativeModule(ivConst)->GetInternModule();
	}
	return pCurModule;
}

IrisClass* IrisInterpreter::GetIrisClass(const string& strClassFullFieldName) {
#if IR_USE_STL_STRING
	list<string> lsRoute;
#else
	list<IrisInternString> lsRoute;
#endif // IR_USE_STL_STRING
	_Split(strClassFullFieldName, lsRoute, "::");
	return GetIrisClass(lsRoute);
}

IrisInterface* IrisInterpreter::GetIrisInterface(const string& strInterfaceFullFieldName) {
#if IR_USE_STL_STRING
	list<string> lsRoute;
#else
	list<IrisInternString> lsRoute;
#endif // IR_USE_STL_STRING
	_Split(strInterfaceFullFieldName, lsRoute, "::");
	return GetIrisInterface(lsRoute);
}

IrisModule* IrisInterpreter::GetIrisModule(const string& strModuleFullFieldName) {
#if IR_USE_STL_STRING
	list<string> lsRoute;
#else
	list<IrisInternString> lsRoute;
#endif // IR_USE_STL_STRING
	_Split(strModuleFullFieldName, lsRoute, "::");
	return GetIrisModule(lsRoute);
}

bool IrisInterpreter::RegistClass(const string& strClassFullFieldName, IIrisClass* pClass, bool bNative) {
#if IR_USE_STL_STRING
	list<string> lsRoute;
#else
	list<IrisInternString> lsRoute;
#endif // IR_USE_STL_STRING
	_Split(strClassFullFieldName, lsRoute, "::");
	return RegistClass(lsRoute, pClass, bNative);
}

bool IrisInterpreter::RegistModule(const string& strModuleFullFieldName, IIrisModule* pModule, bool bNative) {
#if IR_USE_STL_STRING
	list<string> lsRoute;
#else
	list<IrisInternString> lsRoute;
#endif // IR_USE_STL_STRING
	_Split(strModuleFullFieldName, lsRoute, "::");
	return RegistModule(lsRoute, pModule, bNative);
}

bool IrisInterpreter::RegistInterface(const string& strInterfaceFullFieldName, IIrisInterface* pInterface, bool bNative) {
#if IR_USE_STL_STRING
	list<string> lsRoute;
#else
	list<IrisInternString> lsRoute;
#endif // IR_USE_STL_STRING
	_Split(strInterfaceFullFieldName, lsRoute, "::");
	return RegistInterface(lsRoute, pInterface, bNative);
}

#if IR_USE_STL_STRING
bool IrisInterpreter::RegistClass(list<string>& lsRoute, IIrisClass* pClass, bool bNative)
#else
bool IrisInterpreter::RegistClass(list<IrisInternString>& lsRoute, IIrisClass* pClass, bool bNative)
#endif // IR_USE_STL_STRING
{
	if (GetIrisModule(lsRoute)
		|| GetIrisInterface(lsRoute)) {
		// **Error**
		return false;
	}

	auto strRegistName = lsRoute.back();
	lsRoute.pop_back();

	if (bNative) {
		pClass->m_pInternClass = new IrisClass(
			pClass->NativeClassNameDefine(),
			pClass->NativeSuperClassDefine() ? pClass->NativeSuperClassDefine()->GetInternClass() : nullptr,
			pClass->NativeUpperModuleDefine() ? pClass->NativeUpperModuleDefine()->GetInternModule() : nullptr,
			pClass);
	}

	m_iwlClassLock.WriteLock();
	if (!lsRoute.empty()) {
		IrisModule* pModule = GetIrisModule(lsRoute);
		if (!pModule) {
			// **Error**
			m_iwlClassLock.WriteUnlock();
			return false;
		}

		if (pModule->GetClass(strRegistName)) {
			// **Error**
			m_iwlClassLock.WriteUnlock();
			return false;
		}

		pModule->AddClass(pClass->GetInternClass());
		pClass->GetInternClass()->SetUpperModule(pModule);
	}
	m_iwlClassLock.WriteUnlock();

	if (bNative) {
		pClass->NativeClassDefine();
	}

	// ����������
	IrisValue ivObj = IrisValue::WrapObjectPointerToIrisValue(pClass->GetInternClass()->GetClassObject());

	if (pClass->GetInternClass()->GetUpperModule()) {
		pClass->GetInternClass()->GetUpperModule()->AddConstance(strRegistName, ivObj);
	}
	else {
		AddConstance(strRegistName, ivObj);
	}
	return true;
}

#if IR_USE_STL_STRING
bool IrisInterpreter::RegistModule(list<string>& lsRoute, IIrisModule* pModule, bool bNative)
#else
bool IrisInterpreter::RegistModule(list<IrisInternString>& lsRoute, IIrisModule* pModule, bool bNative)
#endif
{
	if (GetIrisClass(lsRoute)
		|| GetIrisInterface(lsRoute)) {
		// **Error**
		return false;
	}

	IrisModule* pTmpModule = nullptr;
	m_iwlModuleLock.WriteLock();
	if (GetIrisModule(lsRoute)) {
		m_iwlModuleLock.WriteUnlock();
		return false;
	}
	auto strRegistName = lsRoute.back();

	if (bNative) {
		pModule->m_pInternModule = new IrisModule(
			pModule->NativeModuleNameDefine(),
			pModule->NativeUpperModuleDefine() ? pModule->NativeUpperModuleDefine()->GetInternModule() : nullptr
			);
		pModule->m_pInternModule->m_pExternModule = pModule;
	}

	lsRoute.pop_back();
	if (!lsRoute.empty()) {
		pTmpModule = GetIrisModule(lsRoute);
		if (pTmpModule && !bNative) {
			pModule->GetInternModule()->m_pUpperModule = pTmpModule;
			pTmpModule->AddInvolvingModule(pModule->GetInternModule());
		}
	}

	m_iwlModuleLock.WriteUnlock();

	if (bNative) {
		pModule->NativeModuleDefine();
	}

	// ���ģ�������
	IrisValue ivObj = IrisValue::WrapObjectPointerToIrisValue(pModule->GetInternModule()->GetModuleObject());
	if (pModule->GetInternModule()->GetUpperModule()) {
		pModule->GetInternModule()->GetUpperModule()->AddConstance(strRegistName, ivObj);
	}
	else {
		AddConstance(strRegistName, ivObj);
	}

	return true;
}

#if IR_USE_STL_STRING
bool IrisInterpreter::RegistInterface(list<string>& lsRoute, IIrisInterface* pInterface, bool bNative)
#else
bool IrisInterpreter::RegistInterface(list<IrisInternString>& lsRoute, IIrisInterface* pInterface, bool bNative)
#endif
{
	if (GetIrisModule(lsRoute)
		|| GetIrisClass(lsRoute)) {
		// **Error**
		return false;
	}

	auto strRegistName = lsRoute.back();
	lsRoute.pop_back();

	m_iwlInterfaceLock.WriteLock();
	if (!lsRoute.empty()) {
		IrisModule* pModule = GetIrisModule(lsRoute);
		if (!pModule) {
			// **Error**
			m_iwlInterfaceLock.WriteUnlock();
			return false;
		}

		if (pModule->GetInterface(strRegistName)) {
			// **Error**
			m_iwlInterfaceLock.WriteUnlock();
			return false;
		}

		pModule->AddInvolvedInterface(pInterface->GetInternInterface());
		pInterface->GetInternInterface()->m_pUpperModule = pModule;
	}
	m_iwlInterfaceLock.WriteUnlock();

	// ����������
	IrisValue ivObj = IrisValue::WrapObjectPointerToIrisValue(pInterface->GetInternInterface()->m_pInterfaceObject);

	if (pInterface->GetInternInterface()->m_pUpperModule) {
		pInterface->GetInternInterface()->m_pUpperModule->AddConstance(strRegistName, ivObj);
	}
	else {
		AddConstance(strRegistName, ivObj);
	}

	return true;
}

#if IR_USE_STL_STRING
IrisClass* IrisInterpreter::GetIrisClass(const list<string>& lsRoute) {
#else
IrisClass* IrisInterpreter::GetIrisClass(const list<IrisInternString>& lsRoute) {
#endif
	IrisClass* pClass = nullptr;
#if IR_USE_STL_STRING
	list<string> lsTmpRoute;
#else
	list<IrisInternString> lsTmpRoute;
#endif

	lsTmpRoute.assign(lsRoute.begin(), lsRoute.end());
	auto strRegistName = lsTmpRoute.back();
	lsTmpRoute.pop_back();

	if (!lsTmpRoute.empty()) {
		m_iwlClassLock.ReadLock();
		IrisModule* pModule = _GetLastModuleFromPath(lsRoute);
		// **Error**
		if (!pModule) {
			m_iwlClassLock.ReadUnlock();
			return nullptr;
		}
		pClass = pModule->GetClass(strRegistName);
		m_iwlClassLock.ReadUnlock();
	}
	else {
		bool bResult = false;
		m_iwlClassLock.ReadLock();
		auto& ivConst = GetConstance(strRegistName, bResult);
		if (!bResult) {
			// **Error**
			m_iwlClassLock.ReadUnlock();
			return nullptr;
		}
		if (!IrisDevUtil::CheckClassIsClass(ivConst) && IrisDevUtil::GetNativeClass(ivConst)->GetInternClass()->GetClassName() != "Class") {
			// **Error**
			m_iwlClassLock.ReadUnlock();
			return nullptr;
		}
		m_iwlClassLock.ReadUnlock();
		pClass = IrisDevUtil::GetNativeClass(ivConst)->GetInternClass();
	}

	return pClass;
}
#if IR_USE_STL_STRING
IrisInterface* IrisInterpreter::GetIrisInterface(const list<string>& lsRoute) {
#else
IrisInterface* IrisInterpreter::GetIrisInterface(const list<IrisInternString>& lsRoute) {
#endif
	IrisInterface* pInterface = nullptr;
#if IR_USE_STL_STRING
	list<string> lsTmpRoute;
#else
	list<IrisInternString> lsTmpRoute;
#endif // IR_USE_STL_STRING

	lsTmpRoute.assign(lsRoute.begin(), lsRoute.end());
	auto strRegistName = lsTmpRoute.back();
	lsTmpRoute.pop_back();

	if (!lsTmpRoute.empty()) {
		m_iwlInterfaceLock.ReadLock();
		IrisModule* pModule = _GetLastModuleFromPath(lsRoute);
		// **Error**
		if (!pModule) {
			m_iwlInterfaceLock.ReadUnlock();
			return nullptr;
		}
		pInterface = pModule->GetInterface(strRegistName);
		m_iwlInterfaceLock.ReadUnlock();
	}
	else {
		bool bResult = false;
		m_iwlInterfaceLock.ReadLock();
		auto& ivConst = GetConstance(strRegistName, bResult);
		if (!bResult) {
			// **Error**
			m_iwlInterfaceLock.ReadUnlock();
			return nullptr;
		}
		if (!IrisDevUtil::CheckClassIsClass(ivConst)) {
			// **Error**
			m_iwlInterfaceLock.ReadUnlock();
			return nullptr;
		}
		m_iwlInterfaceLock.ReadUnlock();
		pInterface = IrisDevUtil::GetNativeInterface(ivConst)->GetInternInterface();
	}

	return pInterface;
}

#if IR_USE_STL_STRING
IrisModule* IrisInterpreter::GetIrisModule(const list<string>& lsRoute) {
#else
IrisModule* IrisInterpreter::GetIrisModule(const list<IrisInternString>& lsRoute) {
#endif // IR_USE_STL_STRING
	IrisModule* pModule = nullptr;
	if (lsRoute.size() == 1) {
		auto& strName = lsRoute.front();
		bool bResult;
		auto& ivConst = GetConstance(strName, bResult);
		if (!bResult) {
			return nullptr;
		}
		if (!IrisDevUtil::CheckClassIsModule(ivConst)) {
			return nullptr;
		}
		pModule = IrisDevUtil::GetNativeModule(ivConst)->GetInternModule();
	}
	else {
		pModule = _GetLastModuleFromPath(lsRoute);
	}
	return pModule;
}

bool IrisInterpreter::AddNewInstanceToHeap(IrisValue& ivValue) {
	lock_guard<recursive_mutex> lgLock(m_rmHeapInsertMT);
	m_hpHeap.AddObject(static_cast<IrisObject*>(ivValue.GetIrisObject()));
	return true;
}

bool IrisInterpreter::AddNewEnvironmentToHeap(IrisContextEnvironment* pEnvironment, IrisThreadInfo* pThreadInfo) {
	//lock_guard<recursive_mutex> lgLock(m_rmHeapInsertMT);
	pThreadInfo->m_ehEnvironmentHeap.insert(pEnvironment);
	return true;
}

bool IrisInterpreter::Run()
{
	if (!m_pCurrentCompiler) {
		return false;
	}

	//auto pThreadInfo = IrisDevUtil::GetCurrentThreadInfo();
	auto pThreadInfo = IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo();
	pThreadInfo->m_nCurrentFileIndex = m_pCurrentCompiler->GetCurrentFileIndex();

	bool bResult = RunCode(m_pCurrentCompiler->GetCodes(), 0, m_pCurrentCompiler->GetCodes().size(), pThreadInfo);

	if (IrregularHappened(pThreadInfo)) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(
			IrisFatalErrorHandler::FatalErrorType::IrregularNotDealedIrregular, 
			pThreadInfo->m_nCurrentLineNumber,
			pThreadInfo->m_nCurrentFileIndex, 
			"There is still an Irregular goaned but NOT being dealed with.", pThreadInfo);
	}

	return bResult;
}

bool IrisInterpreter::RunCode(vector<IR_WORD>& vcVector, unsigned int nStartPointer, unsigned int nEndPointer, IrisThreadInfo* pThreadInfo)
{

	//auto pThreadInfo = IrisDevUtil::GetCurrentThreadInfo();

	//string strOldFileName = pThreadInfo->m_strCurrentFileName;
	//pThreadInfo->m_strCurrentFileName = nCurrentFileIndex;

	//auto nOldFileIndex = pThreadInfo->m_nCurrentFileIndex;
	//pThreadInfo->m_nCurrentFileIndex = nCurrentFileIndex;

	IR_BYTE bInstructor = 0;
	unsigned int nCodePointer = nStartPointer; // 0;
	unsigned int nCodeEnder = nEndPointer; //vcVector.size();
	unsigned int nLineNumber = 0;
	//unsigned int nCurrentFileIndex = 0;
	bool bResult = true;
	while (bResult && nCodePointer != nCodeEnder && !ForceStop(pThreadInfo) && !m_pfExitConditionFunction() && !FatalErrorHappened(pThreadInfo)) {
		// GC
		while (IrisGC::CurrentGC()->sm_bStopTheWorld) {
			unique_lock<mutex> ulWaitLock(IrisGC::CurrentGC()->sm_mtStopTheWorldFinishedMutex);
			IrisThreadManager::CurrentThreadManager()->SetThreadBlock(this_thread::get_id(), true);
			if (IrisThreadManager::CurrentThreadManager()->IsAllThreadBlocked()) {
				IrisGC::CurrentGC()->sm_cvGCStopTheWorldFinishedConditionVariable.notify_all();
			}
			IrisGC::CurrentGC()->sm_cvGCStopTheWorldConditionVariable.wait(ulWaitLock);
			IrisThreadManager::CurrentThreadManager()->SetThreadBlock(this_thread::get_id(), false);
		}

		nLineNumber = vcVector[nCodePointer++];
		pThreadInfo->m_nCurrentLineNumber = nLineNumber;
		//nFileIndex = vcVector[nCodePointer++];
 		bInstructor = vcVector[nCodePointer] >> 8;

		switch (bInstructor)
		{
		case PUSH_ENV: // push_env
 			bResult = push_env(vcVector, nCodePointer, pThreadInfo);
			break;
		case POP_ENV: // pop_env
			bResult = pop_env(vcVector, nCodePointer, pThreadInfo);
			break;
		case PUSH: // push
			bResult = push(vcVector, nCodePointer, pThreadInfo);
			break;
		case POP: // pop
			bResult = pop(vcVector, nCodePointer, pThreadInfo);
			break;
		case CRE_ENV: // cre_env
			bResult = cre_env(vcVector, nCodePointer, pThreadInfo);
			break;
		case LOAD: // load
			bResult = load(vcVector, nCodePointer, pThreadInfo);
			break;
		case NOL_CALL: // nol_call
			bResult = nol_call(vcVector, nCodePointer, pThreadInfo);
			break;
		case ASSIGN: // assign
			bResult = assign(vcVector, nCodePointer, pThreadInfo);
			break;
		case HID_CALL:	// hid_call
			bResult = hid_call(vcVector, nCodePointer, pThreadInfo);
			break;
		case SET_FLD:  // set_fld
			bResult = set_fld(vcVector, nCodePointer, pThreadInfo);
			break;
		case CLR_FLD: // clr_fld
			bResult = clr_fld(vcVector, nCodePointer, pThreadInfo);
			break;
		case FLD_LOAD: // fld_load
			bResult = fld_load(vcVector, nCodePointer, pThreadInfo);
			break;
		case LOAD_NIL: // load_nil
			bResult = load_nil(vcVector, nCodePointer, pThreadInfo);
			break;
		case LOAD_TRUE: // load_true
			bResult = load_true(vcVector, nCodePointer, pThreadInfo);
			break;
		case LOAD_FALSE: // load_false
			bResult = load_false(vcVector, nCodePointer, pThreadInfo);
			break;
		case LOAD_SELF: // load_self
			bResult = load_self(vcVector, nCodePointer, pThreadInfo);
			break;
		case IMTH_DEF: // imth_def
			bResult = imth_def(vcVector, nCodePointer, pThreadInfo);
			break;
		case CMTH_DEF: // cmth_def
			bResult = cmth_def(vcVector, nCodePointer, pThreadInfo);
			break;
		case BLK_DEF: // blk_def
			bResult = blk_def(vcVector, nCodePointer, pThreadInfo);
			break;
		case END_DEF: // end_def
			bResult = end_def(vcVector, nCodePointer, pThreadInfo);
			break;
		case JFON: // jfon
			bResult = jfon(vcVector, nCodePointer, pThreadInfo);
			break;
		case JMP: // jmp
			bResult = jmp(vcVector, nCodePointer, pThreadInfo);
			break;
		case INI_TM: // ini_tm
			bResult = ini_tm(vcVector, nCodePointer, pThreadInfo);
			break;
		case INI_CNT: // ini_cnt
			bResult = ini_cnt(vcVector, nCodePointer, pThreadInfo);
			break;
		case CMP_TAC: // cmp_tac
			bResult = cmp_tac(vcVector, nCodePointer, pThreadInfo);
			break;
		case INC_CNT: // inc_cnt
			bResult = inc_cnt(vcVector, nCodePointer, pThreadInfo);
			break;
		case ASSIGN_LOG: // assign_log
			bResult = assign_log(vcVector, nCodePointer, pThreadInfo);
			break;
		case BRK: // brk
  			bResult = brk(vcVector, nCodePointer, pThreadInfo);
			break;
		case PUSH_DEEP: // push_deep
			bResult = push_deep(vcVector, nCodePointer, pThreadInfo);
			break;
		case POP_DEEP: // pop_deep
			bResult = pop_deep(vcVector, nCodePointer, pThreadInfo);
			break;
		case RTN: // rtn
			bResult = rtn(vcVector, nCodePointer, nEndPointer, pThreadInfo);
			break;
		case CTN: // ctn;
			bResult = ctn(vcVector, nCodePointer, pThreadInfo);
			break;
		case ASSIGN_VSL: // assign_vsl
			bResult = assign_vsl(vcVector, nCodePointer, pThreadInfo);
			break;
		case ASSIGN_ITER: // assign_iter
			bResult = assign_iter(vcVector, nCodePointer, pThreadInfo);
			break;
		case LOAD_ITER: //load_iter
			bResult = load_iter(vcVector, nCodePointer, pThreadInfo);
			break;
		case JT: // jt
			bResult = jt(vcVector, nCodePointer, pThreadInfo);
			break;
		case ASSIGN_CMP: // assign_cmp
			bResult = assign_cmp(vcVector, nCodePointer, pThreadInfo);
			break;
		case CMP_CMP: // cmp_cmp
			bResult = cmp_cmp(vcVector, nCodePointer, pThreadInfo);
			break;
		case CRE_CENV: // cr_cenv
			bResult = cre_cenv(vcVector, nCodePointer, pThreadInfo);
			break;
		case DEF_CLS: // def_cls
			bResult = def_cls(vcVector, nCodePointer, pThreadInfo);
			break;
		case ADD_EXT: // add_ext
			bResult = add_ext(vcVector, nCodePointer, pThreadInfo);
			break;
		case ADD_MLD: // add_mld
			bResult = add_mld(vcVector, nCodePointer, pThreadInfo);
			break;
		case ADD_INF: // add_inf
			bResult = add_inf(vcVector, nCodePointer, pThreadInfo);
			break;
		case PUSH_CNT: // push_cnt
			bResult = push_cnt(vcVector, nCodePointer, pThreadInfo);
			break;
		case PUSH_TIM: // push_tim
			bResult = push_tim(vcVector, nCodePointer, pThreadInfo);
			break;
		case POP_CNT: // pop_cnt
			bResult = pop_cnt(vcVector, nCodePointer, pThreadInfo);
			break;
		case POP_TIM: // pop_tim
			bResult = pop_tim(vcVector, nCodePointer, pThreadInfo);
			break;
		case PUSH_UNIM: // push_unim
			bResult = push_unim(vcVector, nCodePointer, pThreadInfo);
			break;
		case POP_UNIM: // pop_unim
			bResult = pop_unim(vcVector, nCodePointer, pThreadInfo);
			break;
		case PUSH_VSL: // push_vsl
			bResult = push_vsl(vcVector, nCodePointer, pThreadInfo);
			break;
		case POP_VSL: // pop_vsl
			bResult = pop_vsl(vcVector, nCodePointer, pThreadInfo);
			break;
		case PUSH_ITER: // push_iter
			bResult = push_iter(vcVector, nCodePointer, pThreadInfo);
			break;
		case POP_ITER: // pop_iter
			bResult = pop_iter(vcVector, nCodePointer, pThreadInfo);
			break;
		case STR_DEF: // str_def
			bResult = str_def(vcVector, nCodePointer, pThreadInfo);
			break;
		case GTR_DEF: // gtr_def
			bResult = gtr_def(vcVector, nCodePointer, pThreadInfo);
			break;
		case GSTR_DEF: // gstr_def
			bResult = gstr_def(vcVector, nCodePointer, pThreadInfo);
			break;
		case SET_AUTH: // set_auth
			bResult = set_auth(vcVector, nCodePointer, pThreadInfo);
			break;
		case DEF_MLD: // def_mld
			bResult = def_mld(vcVector, nCodePointer, pThreadInfo);
			break;
		case DEF_INF: // def_inf
			bResult = def_inf(vcVector, nCodePointer, pThreadInfo);
			break;
		case DEF_INFS: // def_infs
			bResult = def_infs(vcVector, nCodePointer, pThreadInfo);
			break;
		case CBLK_DEF: // cblk_def
			bResult = cblk_def(vcVector, nCodePointer, pThreadInfo);
			break;
		case BLK: // blk
			bResult = blk(vcVector, nCodePointer, pThreadInfo);
			break;
		//case 62: // cast
		//	bResult = cast(vcVector, nCodePointer);
		//	break;
		case REG_IRP: // reg_irp
			bResult = reg_irp(vcVector, nCodePointer, pThreadInfo);
			break;
		case UREG_IRP: // ureg_irp
			bResult = ureg_irp(vcVector, nCodePointer, pThreadInfo);
			break;
		case ASSIGN_IR: // assign_ir
			bResult = assign_ir(vcVector, nCodePointer, pThreadInfo);
			break;
		case GRN: // grn
			bResult = grn(vcVector, nCodePointer, pThreadInfo);
			break;
		case SPR: // spr
			bResult = spr(vcVector, nCodePointer, pThreadInfo);
		case LOAD_CAST:
			bResult = load_cast(vcVector, nCodePointer, pThreadInfo);
			break;
		default:
			break;
		}
		++nCodePointer;
	}

	--nCodePointer;

	//pThreadInfo->m_nCurrentFileIndex = nOldFileIndex;

	// Irregular
	if (IrregularHappened(pThreadInfo) || FatalErrorHappened(pThreadInfo)) {
		rtn(vcVector, nCodePointer, nEndPointer, pThreadInfo);
		return false;
	}

	return bResult;
}

IrisAM IrisInterpreter::GetOneAM(vector<IR_WORD>& vcVector, unsigned int& nCodePointer)
{
	IR_BYTE bType = (IR_BYTE)vcVector[++nCodePointer];
	IR_WORD wLow = vcVector[++nCodePointer];
	IR_WORD wHigh = vcVector[++nCodePointer];
	return IrisAM(bType, (((0x00000000) | wHigh) << 16) | wLow);
}

bool IrisInterpreter::LoadExtension(const string & strExtensionPath)
{	
	if (m_emLoadedExtention.find(strExtensionPath) != m_emLoadedExtention.end()) {
		return true;
	}

	HMODULE hMod = ::LoadLibraryA(strExtensionPath.c_str());
	if (!hMod) {
		::FreeLibrary(hMod);
		return false;
	}

	typedef bool(*InitFunc)();

	InitFunc pfInitFunc = (InitFunc)GetProcAddress(hMod, "IR_Initialize");
 	if (!pfInitFunc) {
		return false;
	}
	if (!pfInitFunc()) {
		return false;
	}
	
	m_emLoadedExtention.insert(_ExtentionPair(strExtensionPath, hMod));

	return true;
}

IrisInterpreter::~IrisInterpreter()
{
}

void IrisInterpreter::PopStack(unsigned int nTimes, IrisThreadInfo* pThreadInfo)
{
	for (size_t i = 0; i < nTimes; ++i) {
		pThreadInfo->m_stStack.Pop();
	}
}

#if IR_USE_STL_STRING
inline IrisMethod * IrisInterpreter::GetMainMethod(const string & strMethodName) {
#else
inline IrisMethod * IrisInterpreter::GetMainMethod(const IrisInternString & strMethodName) {
#endif // IR_USE_STL_STRING
	m_iwlMethodLock.ReadLock();
	decltype(m_mpMethods)::iterator iMethod;
	if ((iMethod = m_mpMethods.find(strMethodName)) == m_mpMethods.end()) {
		m_iwlMethodLock.ReadUnlock();
		return nullptr;
	}
	else {
		auto& pResult = iMethod->second;
		m_iwlMethodLock.ReadUnlock();
		return pResult;
	}
}

#if IR_USE_STL_STRING
inline void IrisInterpreter::AddMainMethod(const string & strMethodName, IrisMethod * pMethod) {
#else
inline void IrisInterpreter::AddMainMethod(const IrisInternString & strMethodName, IrisMethod * pMethod) {
#endif // IR_USE_STL_STRING
	m_iwlMethodLock.WriteLock();
	decltype(m_mpMethods)::iterator iMethod;
	if ((iMethod = m_mpMethods.find(strMethodName)) == m_mpMethods.end()) {
		m_mpMethods.insert(_MethodPair(strMethodName, pMethod));
	}
	else {
		//delete iMethod->second;
		m_mpMethods[strMethodName] = pMethod;
	}
	m_iwlMethodLock.WriteUnlock();
}

// -------------- Instructors --------------

void IrisInterpreter::GetCodesFromBlock(unsigned int nIndex, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisCodeSegment& icsCodeSegment) {

	nCodePointer += 2;

	icsCodeSegment.m_nStartPointer = nCodePointer - 1;
	icsCodeSegment.m_pWholeCodes = &vcVector;

	IrisAM iaAM;
	IR_WORD wVirtualCode = vcVector[nCodePointer];
	IR_BYTE bOperators = 0;

	// �� end_def Ϊֹ�������е�virtual code�ŵ�BlockCodes��
	//unsigned int nLineNumber = vcVector[nCodePointer - 2];
	//unsigned int nCurrentFileIndex = vcVector[nCodePointer - 1];
	while (true) {
		if (wVirtualCode >> 8 == 19) {
			iaAM = GetOneAM(vcVector, nCodePointer);
			if (iaAM.m_dwIndex == nIndex) {
				icsCodeSegment.m_nEndPointer = nCodePointer - 4;
				break;
			}
		}
		else {
			bOperators = (IR_BYTE)(wVirtualCode & 0x00FF);
			nCodePointer += 3 * bOperators;
		}
		++nCodePointer;
		//nLineNumber = vcVector[++nCodePointer];
		//nFileIndex = vcVector[++nCodePointer];
		wVirtualCode = vcVector[++nCodePointer];
	}
}

#if IR_USE_STL_STRING
bool IrisInterpreter::BuildUserFunction(void** pFunction, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, string& strMethodName, IrisThreadInfo* pThreadInfo) {
#else
bool IrisInterpreter::BuildUserFunction(void** pFunction, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisInternString& strMethodName, IrisThreadInfo* pThreadInfo) {
#endif // IR_USE_STL_STRING
	
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	
	IrisCompiler* pCompiler = m_pCurrentCompiler;

	IrisMethod::UserFunction* pUserFunction = new IrisMethod::UserFunction();
	IR_DWORD dwParameterCount = (vcVector[nCodePointer] & 0x00FF) - 4;

	// Name
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	strMethodName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	// Parameter
	for (int i = 0; i < dwParameterCount; ++i) {
		iaAM = GetOneAM(vcVector, nCodePointer);
		pUserFunction->m_lsParameters.push_back(pCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex));
	}

	// VariableParameter
	iaAM = GetOneAM(vcVector, nCodePointer);
	pUserFunction->m_strVariableParameter = iaAM.m_dwIndex == -1 ? "" : pCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	// WithCastBlock
	iaAM = GetOneAM(vcVector, nCodePointer);
	bool bWithCastBlock = iaAM.m_dwIndex == 1;

	iaAM = GetOneAM(vcVector, nCodePointer);
	unsigned int nIndex = iaAM.m_dwIndex;
	pUserFunction->m_dwIndex = nIndex;

	// Block
	// �����һ��ָ�Ϊblk_def�򱨴�
	++nCodePointer;
	IR_BYTE bInstructor = vcVector[++nCodePointer] >> 8;
	if (bInstructor != 18) {
		// ** Error **
		return false;
	}
	iaAM = GetOneAM(vcVector, nCodePointer);
	nIndex = iaAM.m_dwIndex;
	
	GetCodesFromBlock(nIndex, vcVector, nCodePointer, pUserFunction->m_icsBlockCodes);
	pUserFunction->m_icsBlockCodes.m_nBelongingFileIndex = nCurrentFileIndex;

	// ����� with block ����һ����䲻Ϊblk_def �򱨴�
	if (bWithCastBlock) {
		++nCodePointer;
		bInstructor = vcVector[++nCodePointer] >> 8;
		if (bInstructor != 18) {
			// ** Error **
			return false;
		}
		else {
			iaAM = GetOneAM(vcVector, nCodePointer);
			nIndex = iaAM.m_dwIndex;
			// with block
			GetCodesFromBlock(nIndex, vcVector, nCodePointer, pUserFunction->m_icsWithBlockCodes);
			pUserFunction->m_icsWithBlockCodes.m_nBelongingFileIndex = nCurrentFileIndex;
			// without block
			// skip blk_def
			nCodePointer += 2;

			iaAM = GetOneAM(vcVector, nCodePointer);
			nIndex = iaAM.m_dwIndex;
			GetCodesFromBlock(nIndex, vcVector, nCodePointer, pUserFunction->m_icsWithoutBlockCodes);
			pUserFunction->m_icsWithoutBlockCodes.m_nBelongingFileIndex = nCurrentFileIndex;
		}
	}

	*pFunction = pUserFunction;
	return true;
}

bool IrisInterpreter::push_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_skEnvironmentStack.push_back(pThreadInfo->m_pEnvrionmentRegister);
	return true;
}

bool IrisInterpreter::pop_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_pEnvrionmentRegister = pThreadInfo->m_skEnvironmentStack.back();
	pThreadInfo->m_skEnvironmentStack.pop_back();
	return true;
}

bool IrisInterpreter::push(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_stStack.Push(pThreadInfo->m_ivResultRegister);
	return true;
}

bool IrisInterpreter::pop(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);

	if (IrisDevUtil::IrregularHappened(pThreadInfo) && iaAM.m_dwIndex > static_cast<int>(pThreadInfo->m_stStack.m_lsStack.size())) {
		pThreadInfo->m_stStack.m_lsStack.clear();
	}
	else {
		for (int i = 0; i < iaAM.m_dwIndex; ++i) {
			pThreadInfo->m_stStack.Pop();
		}
	}
	return true;
}

bool IrisInterpreter::cre_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto pNewContextEnvironment = new IrisContextEnvironment();
	pNewContextEnvironment->m_pUpperContextEnvironment = pThreadInfo->m_pEnvrionmentRegister;
	pNewContextEnvironment->m_eType = IrisContextEnvironment::EnvironmentType::Runtime;
	pThreadInfo->m_pEnvrionmentRegister = pNewContextEnvironment;
	//IrisGC::AddContextEnvironmentSize();
	//IrisGC::ContextEnvironmentGC();
	AddNewEnvironmentToHeap(pNewContextEnvironment, pThreadInfo);
	return true;
}

bool IrisInterpreter::load(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	IrisAMType iatType = (IrisAMType)iaAM.m_bType;
	switch (iatType)
	{
	case IrisAMType::ImmediateString:
	{
		auto& strString = m_pCurrentCompiler->GetString(iaAM.m_dwIndex, nCurrentFileIndex);
		//pThreadInfo->m_ivResultRegister = GetIrisClass("String")->CreateInstanceByInstantValue(strString);
#if IR_USE_STL_STRING
		pThreadInfo->m_ivResultRegister = IrisDevUtil::CreateInstanceByInstantValue(strString.c_str());
#else
		pThreadInfo->m_ivResultRegister = IrisDevUtil::CreateInstanceByInstantValue(strString.GetCTypeString());
#endif
	}
		break;
	case IrisAMType::ImmediateUniqueString:
	{
		//const string& strString = m_pCurrentCompiler->GetString(iaAM.m_dwIndex);
		//pThreadInfo->m_ivResultRegister =  static_cast<IrisUniqueString*>(GetIrisClass("UniqueString"))->CreateInstanceByUniqueIndex(iaAM.m_dwIndex);
		pThreadInfo->m_ivResultRegister = IrisDevUtil::CreateUniqueStringInstanceByUniqueIndex(iaAM.m_dwIndex);
	}
		break;
	case IrisAMType::ImmediateInteger: 
	{
		int nInteger = m_pCurrentCompiler->GetInteger(iaAM.m_dwIndex, nCurrentFileIndex);
		//pThreadInfo->m_ivResultRegister = GetIrisClass("Integer")->CreateInstanceByInstantValue(nInteger);
		pThreadInfo->m_ivResultRegister = IrisDevUtil::CreateInstanceByInstantValue(nInteger);
	}
		break;
	case IrisAMType::ImmediateFloat:
	{
		double dFloat = m_pCurrentCompiler->GetFloat(iaAM.m_dwIndex, nCurrentFileIndex);
		//pThreadInfo->m_ivResultRegister = GetIrisClass("Float")->CreateInstanceByInstantValue(dFloat);
		pThreadInfo->m_ivResultRegister = IrisDevUtil::CreateInstanceByInstantValue(dFloat);
	}
		break;
	case IrisAMType::Constance:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		// Main
		if (!pThreadInfo->m_pEnvrionmentRegister || pThreadInfo->m_pEnvrionmentRegister->m_bIsThreadMainContext || pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure) {
			pThreadInfo->m_ivResultRegister = GetConstance(strIdentifier, bResult);
			if (!bResult) {
				AddConstance(strIdentifier, m_ivNil);
			}
		}
		// Class
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ClassDefineTime) {
			IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
			pThreadInfo->m_ivResultRegister = pClass->SearchConstance(strIdentifier, bResult);
			if (!bResult) {
				pClass->AddConstance(strIdentifier, m_ivNil);
			}
		}
		// Module
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ModuleDefineTime) {
			IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
			pThreadInfo->m_ivResultRegister = pModule->SearchConstance(strIdentifier, bResult);
			if (!bResult) {
				pModule->AddConstance(strIdentifier, m_ivNil);
			}
		}
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::Runtime) {
			IrisObject* pObject = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject;
			pThreadInfo->m_ivResultRegister = pObject->GetClass()->GetInternClass()->SearchConstance(strIdentifier, bResult);
			if (!bResult) {
				// ** Error **
#if IR_USE_STL_STRING
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdentifierNotFoundIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Identifier of " + strIdentifier + " not found.", pThreadInfo);
#else
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdentifierNotFoundIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Identifier of " + strIdentifier.GetSTLString() + " not found.", pThreadInfo);
#endif // IR_USE_STL_STRING
				return false;
			}
		}
		else {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ConstanceDeclareIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Constance can not be declared here.", pThreadInfo);
			return false;
		}
	}
		break;
	case IrisAMType::GlobalValue:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		pThreadInfo->m_ivResultRegister = GetGlobalValue(strIdentifier, bResult);
		if (!bResult) {
			AddGlobalValue(strIdentifier, m_ivNil);
		}
	}
		break;
	case IrisAMType::ClassValue:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		if (!pThreadInfo->m_pEnvrionmentRegister) {
			pThreadInfo->m_ivResultRegister = GetOtherValue(strIdentifier, bResult);
			if (!bResult) {
				AddOtherValue(strIdentifier, m_ivNil);
			}
		}
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ClassDefineTime) {
			IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
			pThreadInfo->m_ivResultRegister = pClass->SearchClassVariable(strIdentifier, bResult);
			if (!bResult) {
				pClass->AddClassVariable(strIdentifier, m_ivNil);
			}
		}
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ModuleDefineTime) {
			IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
			pThreadInfo->m_ivResultRegister = pModule->SearchClassVariable(strIdentifier, bResult);
			if (!bResult) {
				pModule->AddClassVariable(strIdentifier, m_ivNil);
			}
		}
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::InterfaceDefineTime) {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ClassVariableDeclareIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Class variable cannot be declared in interface.", pThreadInfo);
			return false;
		}
		// Runtime
		else {
			IrisObject* pObject = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject;
			if (pObject) {
				if (pObject->GetClass()->GetInternClass()->IsClassClass()) {
					pThreadInfo->m_ivResultRegister = (IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(pObject))->GetClass()->SearchClassVariable(strIdentifier, bResult);
					if (!bResult) {
						IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(pObject)->GetClass()->AddClassVariable(strIdentifier, m_ivNil);
					}
				}
				else if (pObject->GetClass()->GetInternClass()->IsModuleClass()) {
					pThreadInfo->m_ivResultRegister = IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>(pObject)->GetModule()->SearchClassVariable(strIdentifier, bResult);
					if (!bResult) {
						IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>(pObject)->GetModule()->AddClassVariable(strIdentifier, m_ivNil);
					}
				}
				else {
					pThreadInfo->m_ivResultRegister = pObject->GetClass()->GetInternClass()->SearchClassVariable(strIdentifier, bResult);
					if (!bResult) {
						pObject->GetClass()->GetInternClass()->AddClassVariable(strIdentifier, m_ivNil);
					}
				}
			}
			// Main
			else {
				pThreadInfo->m_ivResultRegister = pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
					? pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetClassVariable(strIdentifier,bResult)
					: GetOtherValue(strIdentifier, bResult);
				if (!bResult) {
					pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
						? pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->AddOtherVariable(strIdentifier, m_ivNil)
						: AddOtherValue(strIdentifier, m_ivNil);
				}
			}
		}
	}
		break;
	case IrisAMType::InstanceValue:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		IrisObject* pObject = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject;
		if (pObject) {
			pThreadInfo->m_ivResultRegister = pObject->GetInstanceValue(strIdentifier, bResult);
			if (!bResult) {
				pObject->AddInstanceValue(strIdentifier, m_ivNil);
			}
		}
		// Main
		else {
			pThreadInfo->m_ivResultRegister = pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
				? pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetInstanceVariable(strIdentifier, bResult)
				: GetOtherValue(strIdentifier, bResult);
			if (!bResult) {
				pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
					? pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->AddOtherVariable(strIdentifier, m_ivNil)
					: AddOtherValue(strIdentifier, m_ivNil);
			}
		}
	}
		break;
	case IrisAMType::LocalValue:
		// Main
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		if (!pThreadInfo->m_pEnvrionmentRegister) {
			//|| pThreadInfo->m_pEnvrionmentRegister->m_eType != IrisContextEnvironment::EnvironmentType::Runtime) {
			pThreadInfo->m_ivResultRegister = GetOtherValue(strIdentifier, bResult);
			if (!bResult) {
				AddOtherValue(strIdentifier, m_ivNil);
			}
		}
		else if(pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::Runtime) {
			pThreadInfo->m_ivResultRegister = pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
				? pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetLocalVariable(strIdentifier, bResult)
				: pThreadInfo->m_pEnvrionmentRegister->GetVariableValue(strIdentifier, bResult);
			if (!bResult) {
				pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
					? pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->AddLocalVariable(strIdentifier, m_ivNil)
					: pThreadInfo->m_pEnvrionmentRegister->AddLocalVariable(strIdentifier, m_ivNil);
			}
		}
	}
		break;
	case IrisAMType::MemberValue:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
#if IR_USE_STL_STRING
		string strGetterMethod = "__get_" + strIdentifier;
#else
		string strGetterMethod = "__get_" + strIdentifier.GetSTLString();
#endif // IR_USE_STL_STRING
		pThreadInfo->m_ivResultRegister = static_cast<IrisObject*>(pThreadInfo->m_ivResultRegister.GetIrisObject())->CallInstanceFunction(strGetterMethod, nullptr, nullptr, pThreadInfo, CallerSide::Outside);
	}
		break;
//	case IrisAMType::SelfMemberValue:
//	{
//		bool bResult = false;
//		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
//#if IR_USE_STL_STRING
//		string strInstanceVariableName = "@" + strIdentifier;
//#else
//		string strInstanceVariableName = "@" + strIdentifier.GetSTLString();
//#endif // IR_USE_STL_STRING
//		pThreadInfo->m_ivResultRegister = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject->GetInstanceValue(strInstanceVariableName, bResult);
//		if (!bResult) {
//			pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject->AddInstanceValue(strInstanceVariableName, m_ivNil);
//		}
//	}
//		break;
	case IrisAMType::IndexValue:
	{
		bool bResult = false;
		IrisValues ivsValues = { pThreadInfo->m_stStack.m_lsStack.back() };
		pThreadInfo->m_ivResultRegister = static_cast<IrisObject*>(pThreadInfo->m_ivResultRegister.GetIrisObject())->CallInstanceFunction("[]",  &ivsValues, nullptr, pThreadInfo, CallerSide::Outside);
	}
		break;
	case IrisAMType::RegistValue:
	case IrisAMType::Identifier:
	case IrisAMType::Extends:
	default:
		break;
	}
	return true;
}

bool IrisInterpreter::nol_call(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	if (pThreadInfo->m_pClosureBlockRegister) {
		pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock = pThreadInfo->m_pClosureBlockRegister;
		pThreadInfo->m_pClosureBlockRegister = nullptr;

		auto ivObj = IrisDevUtil::CreateInstanceByInstantValue(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock);
		pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->SetNativeObject(static_cast<IrisObject*>(ivObj.GetIrisObject()));

		IrisContextEnvironment* pTmpEnv = static_cast<IrisContextEnvironment*>(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetCurrentContextEnvironment());
		while (pTmpEnv) {
			++pTmpEnv->m_nReferenced;
			pTmpEnv = static_cast<IrisContextEnvironment*>(pTmpEnv->GetUpperContextEnvrioment());
		}
	}

	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	auto& strMethodName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	iaAM = GetOneAM(vcVector, nCodePointer);
	const int nParameters = iaAM.m_dwIndex;

	IrisValues ivsValues(nParameters);
	int i = 0;

	ivsValues.GetVector().assign(pThreadInfo->m_stStack.m_lsStack.begin() + pThreadInfo->m_stStack.m_lsStack.size() - nParameters, pThreadInfo->m_stStack.m_lsStack.end());

	pThreadInfo->m_stStack.Push(pThreadInfo->m_ivResultRegister);
	pThreadInfo->m_ivResultRegister = static_cast<IrisObject*>(pThreadInfo->m_ivResultRegister.GetIrisObject())->CallInstanceFunction(strMethodName, &ivsValues, pThreadInfo->m_pEnvrionmentRegister, pThreadInfo, CallerSide::Outside);
	pThreadInfo->m_stStack.Pop();

	if (pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock) {
		IrisContextEnvironment* pTmpEnv = static_cast<IrisContextEnvironment*>(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetCurrentContextEnvironment());
		while (pTmpEnv) {
			--pTmpEnv->m_nReferenced;
			pTmpEnv = pTmpEnv->m_pUpperContextEnvironment;
		}
	}

	if (IrregularHappened(pThreadInfo)) {
		PopEnvironment(pThreadInfo);
	}

	return true;
}

bool IrisInterpreter::assign(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	IrisAMType iatType = (IrisAMType)iaAM.m_bType;
	switch (iatType)
	{
	case IrisAMType::ImmediateString:
	case IrisAMType::ImmediateInteger:
	case IrisAMType::ImmediateFloat:
		// ** Error **
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::LeftValueIrregular,
			pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
			"Left value must be IDENTIFIER, object's MEMBER, self MEMBER or array/hash's INDEX.", pThreadInfo);
		return false;
		break;
	case IrisAMType::Constance:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		// if exist then error
		// Main
		if (!pThreadInfo->m_pEnvrionmentRegister) {
			GetConstance(strIdentifier, bResult);
			if (!bResult) {
				AddConstance(strIdentifier, pThreadInfo->m_ivResultRegister);
			}
			else {
				// ** Error **
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ConstanceReassignIrregular,
					pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
					"Constance can not be reassigned.", pThreadInfo);
				return false;
			}
		}
		// Class
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ClassDefineTime) {
			IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
			pClass->SearchConstance(strIdentifier, bResult);
			if (!bResult) {
				pClass->AddConstance(strIdentifier, pThreadInfo->m_ivResultRegister);
			}
			else {
				// ** Error **
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ConstanceReassignIrregular,
					pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
					"Constance can not be reassigned.", pThreadInfo);
				return false;
			}
		}
		// Module
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ModuleDefineTime) {
			IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
			pModule->SearchConstance(strIdentifier, bResult);
			if (!bResult) {
				pModule->AddConstance(strIdentifier, pThreadInfo->m_ivResultRegister);
			}
			else {
				// ** Error **
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ConstanceReassignIrregular,
					pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
					"Constance can not be reassigned.", pThreadInfo);
				return false;
			}
		}
		else {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ConstanceDeclareIrregular,
				pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
				"Constance can not be declared here.", pThreadInfo);
			return false;
		}
	}
	break;
	case IrisAMType::GlobalValue:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		IrisValue& ivTmp = (IrisValue&)GetGlobalValue(strIdentifier, bResult);
		if (!bResult) {
			AddGlobalValue(strIdentifier, pThreadInfo->m_ivResultRegister);
		}
		else {
			ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
		}
	}
	break;
	case IrisAMType::ClassValue:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		if (!pThreadInfo->m_pEnvrionmentRegister) {
			IrisValue& ivTmp = (IrisValue&)GetOtherValue(strIdentifier, bResult);
			if (!bResult) {
				AddOtherValue(strIdentifier, pThreadInfo->m_ivResultRegister);
			}
			else {
				ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
			}
		}
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ClassDefineTime) {
			IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
			IrisValue& ivTmp = (IrisValue&)pClass->SearchClassVariable(strIdentifier, bResult);
			if (!bResult) {
				pClass->AddClassVariable(strIdentifier, pThreadInfo->m_ivResultRegister);
			}
			else {
				ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
			}
		}
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ModuleDefineTime) {
			IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
			IrisValue& ivTmp = (IrisValue&)pModule->SearchClassVariable(strIdentifier, bResult);
			if (!bResult) {
				pModule->AddClassVariable(strIdentifier, pThreadInfo->m_ivResultRegister);
			}
			else {
				ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
			}
		}
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::InterfaceDefineTime) {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ClassVariableDeclareIrregular,
				pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
				"Class variable cannot be declared in interface.", pThreadInfo);
			return false;
		}
		// Runtime
		else {
			IrisObject* pObject = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject;
			if (pObject) {
				if (pObject->GetClass()->GetInternClass()->IsClassClass()) {
					IrisValue& ivTmp = (IrisValue&)((IrisClassBaseTag*)(pObject->GetNativeObject()))->GetClass()->SearchClassVariable(strIdentifier, bResult);
					if (!bResult) {
						((IrisClassBaseTag*)(pObject->GetNativeObject()))->GetClass()->AddClassVariable(strIdentifier, pThreadInfo->m_ivResultRegister);
					}
					else {
						ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
					}
				}
				else if (pObject->GetClass()->GetInternClass()->IsModuleClass()) {
					IrisValue& ivTmp = (IrisValue&)((IrisModuleBaseTag*)(pObject->GetNativeObject()))->GetModule()->SearchClassVariable(strIdentifier, bResult);
					if (!bResult) {
						((IrisModuleBaseTag*)(pObject->GetNativeObject()))->GetModule()->AddClassVariable(strIdentifier, pThreadInfo->m_ivResultRegister);
					}
					else {
						ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
					}
				}
				else {
					IrisValue& ivTmp = (IrisValue&)pObject->GetClass()->GetInternClass()->SearchClassVariable(strIdentifier, bResult);
					if (!bResult) {
						pObject->GetClass()->GetInternClass()->AddClassVariable(strIdentifier, pThreadInfo->m_ivResultRegister);
					}
					else {
						ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
					}
				}
			}
			// Main
			else {
				IrisValue& ivTmp = pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
					? (IrisValue&)pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetClassVariable(strIdentifier,bResult)
					: (IrisValue&)GetOtherValue(strIdentifier, bResult);
				if (!bResult) {
					pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure ?
						pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->AddOtherVariable(strIdentifier, pThreadInfo->m_ivResultRegister)
						: AddOtherValue(strIdentifier, pThreadInfo->m_ivResultRegister);
				}
				else {
					ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
				}
			}
		}
	}
		break;
	case IrisAMType::InstanceValue:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		if (!pThreadInfo->m_pEnvrionmentRegister) {
			//|| pThreadInfo->m_pEnvrionmentRegister->m_eType != IrisContextEnvironment::EnvironmentType::Runtime) {
			IrisValue& ivTmp = (IrisValue&)GetOtherValue(strIdentifier, bResult);
			if (!bResult) {
				AddOtherValue(strIdentifier, pThreadInfo->m_ivResultRegister);
			}
			else {
				ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
			}
		}
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::Runtime) {
			IrisObject* pObject = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject;
			if (pObject) {
				//if (pObject->GetClass()->GetInternClass()->IsClassClass() || pObject->GetClass()->GetInternClass()->IsModuleClass()) {
				//	IrisValue& ivTmp = (IrisValue&)GetOtherValue(strIdentifier, bResult);
				//	if (!bResult) {
				//		AddOtherValue(strIdentifier, pThreadInfo->m_ivResultRegister);
				//	}
				//	else {
				//		ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
				//	}
				//}
				//else {
					IrisValue& ivTmp = (IrisValue&)pObject->GetInstanceValue(strIdentifier, bResult);
					if (!bResult) {
						pObject->AddInstanceValue(strIdentifier, pThreadInfo->m_ivResultRegister);
					}
					else {
						ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
					}
				//}
			}
			// Main
			else {
				IrisValue& ivTmp = pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
					? (IrisValue&)pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetInstanceVariable(strIdentifier, bResult)
					: (IrisValue&)GetOtherValue(strIdentifier, bResult);
				if (!bResult) {
					pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure ?
						pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->AddOtherVariable(strIdentifier, pThreadInfo->m_ivResultRegister)
						: AddOtherValue(strIdentifier, pThreadInfo->m_ivResultRegister);
				}
				else {
					ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
				}
			}
		}
	}
		break;
	case IrisAMType::LocalValue:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
		if (!pThreadInfo->m_pEnvrionmentRegister) {
			//|| pThreadInfo->m_pEnvrionmentRegister->m_eType != IrisContextEnvironment::EnvironmentType::Runtime) {
			IrisValue& ivTmp = (IrisValue&)GetOtherValue(strIdentifier, bResult);
			if (!bResult) {
				AddOtherValue(strIdentifier, pThreadInfo->m_ivResultRegister);
			}
			else {
				ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
			}
		}
		else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::Runtime) {
			IrisValue& ivTmp = pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
				? (IrisValue&)pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetLocalVariable(strIdentifier, bResult)
				: (IrisValue&)pThreadInfo->m_pEnvrionmentRegister->GetVariableValue(strIdentifier, bResult);
			if (!bResult) {
				pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure
					? pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->AddLocalVariable(strIdentifier, pThreadInfo->m_ivResultRegister)
					: pThreadInfo->m_pEnvrionmentRegister->AddLocalVariable(strIdentifier, pThreadInfo->m_ivResultRegister);
			}
			else {
				ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
			}
		}
	}
		break;
	case IrisAMType::MemberValue:
	{
		bool bResult = false;
		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
#if IR_USE_STL_STRING
		string strSetterName = "__set_" + strIdentifier;
#else
		string strSetterName = "__set_" + strIdentifier.GetSTLString();
#endif // IR_USE_STL_STRING
		IrisValues ivsValues = { pThreadInfo->m_stStack.m_lsStack.back() };
		static_cast<IrisObject*>(pThreadInfo->m_ivResultRegister.GetIrisObject())->CallInstanceFunction(strSetterName, &ivsValues, nullptr, pThreadInfo, CallerSide::Outside);
	}
		break;
//	case IrisAMType::SelfMemberValue:
//	{
//		bool bResult = false;
//		auto& strIdentifier = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
//		if (!pThreadInfo->m_pEnvrionmentRegister || !pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject) {
//			// ** Error **
//			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::SelfPointerIrregular,
//				pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
//				"Statement of self must be used in INSTANCE METHOD OF CLASS.");
//			return false;
//		}
//		else {
//			bool bResult = false;
//			IrisObject* pObject = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject;
//#if IR_USE_STL_STRING
//			string strIntanceVaraible = "@" + strIdentifier;
//#else
//			string strIntanceVaraible = "@" + strIdentifier.GetSTLString();
//#endif // IR_USE_STL_STRING
//			IrisValue& ivTmp = (IrisValue&)pObject->GetInstanceValue(strIntanceVaraible, bResult);
//			if (!bResult) {
//				pObject->AddInstanceValue(strIntanceVaraible, pThreadInfo->m_ivResultRegister);
//			}
//			else {
//				ivTmp.SetIrisObject(pThreadInfo->m_ivResultRegister.GetIrisObject());
//			}
//		}
//	}
//		break;
	case IrisAMType::IndexValue:
	{
		bool bResult = false;
		auto it = pThreadInfo->m_stStack.m_lsStack.rbegin();
		IrisValue ivKey = *pThreadInfo->m_stStack.m_lsStack.rbegin();
		IrisValue ivValue = *(pThreadInfo->m_stStack.m_lsStack.rbegin() + 1);
		IrisValues ivsValues = { ivKey, ivValue };
		static_cast<IrisObject*>(pThreadInfo->m_ivResultRegister.GetIrisObject())->CallInstanceFunction("[]=", &ivsValues, nullptr, pThreadInfo, CallerSide::Outside);
	}
		break;
	case IrisAMType::RegistValue:
		break;
	case IrisAMType::Identifier:
		break;
	case IrisAMType::Extends:
		break;
	default:
		break;
	}
	return true;
}

bool IrisInterpreter::hid_call(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	if (pThreadInfo->m_pClosureBlockRegister) {
		pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock = pThreadInfo->m_pClosureBlockRegister;
		pThreadInfo->m_pClosureBlockRegister = nullptr;

		auto ivObj = IrisDevUtil::CreateInstanceByInstantValue(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock);
		pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->SetNativeObject(static_cast<IrisObject*>(ivObj.GetIrisObject()));

		IrisContextEnvironment* pTmpEnv = static_cast<IrisContextEnvironment*>(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetCurrentContextEnvironment());
		while (pTmpEnv) {
			++pTmpEnv->m_nReferenced;
			pTmpEnv = pTmpEnv->m_pUpperContextEnvironment;
		}
	}

	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	auto& strMethodName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	iaAM = GetOneAM(vcVector, nCodePointer);
	const int nParameters = iaAM.m_dwIndex;

	IrisValues* pValues = nullptr;
	IrisValues ivsValues(nParameters);
	if (nParameters) {
		int i = 0;
		//for (auto it = m_stStack.m_lsStack.rbegin(); i < nParameters; ++i, ++it) {
		//	ivsValues[nParameters - i - 1] = *it;
		//}
		ivsValues.GetVector().assign(pThreadInfo->m_stStack.m_lsStack.begin() + pThreadInfo->m_stStack.m_lsStack.size() - nParameters, pThreadInfo->m_stStack.m_lsStack.end());
		pValues = &ivsValues;
	}

	if (pThreadInfo->m_pEnvrionmentRegister->m_eType != IrisContextEnvironment::EnvironmentType::Runtime) {
		IrisMethod* pMethod = GetMainMethod(strMethodName);
		if (pMethod) {
			pThreadInfo->m_ivResultRegister = pMethod->CallMainMethod(pValues, pThreadInfo);
		}
		else {
			pThreadInfo->m_ivResultRegister = GetIrisModule("Kernel")->CallClassMethod(strMethodName, pValues, pThreadInfo->m_pEnvrionmentRegister, pThreadInfo, CallerSide::Outside);
		}
	}
	else {
		IrisContextEnvironment* pUpperEnvironment = pThreadInfo->m_pEnvrionmentRegister->m_pUpperContextEnvironment;
		if(pUpperEnvironment && pUpperEnvironment->m_uType.m_pCurObject) {
			IrisObject* pObject = pUpperEnvironment->m_uType.m_pCurObject;
			//if (pObject->GetClass()->GetInternClass()->IsClassClass()) {
			//	pThreadInfo->m_ivResultRegister = ((IrisClassBaseTag*)pObject->GetNativeObject())->GetClass()->CallClassMethod(strMethodName, pThreadInfo->m_pEnvrionmentRegister, pValues, CallerSide::Inside);
			//}
			//else if (pObject->GetClass()->GetInternClass()->IsModuleClass()) {
			//	pThreadInfo->m_ivResultRegister = ((IrisModuleBaseTag*)pObject->GetNativeObject())->GetModule()->CallClassMethod(strMethodName, pThreadInfo->m_pEnvrionmentRegister, pValues, CallerSide::Inside);
			//}
			//else {
				pThreadInfo->m_ivResultRegister = pObject->CallInstanceFunction(strMethodName, pValues, pThreadInfo->m_pEnvrionmentRegister, pThreadInfo, CallerSide::Inside);
			//}
		}
		else {
			IrisMethod* pMethod = GetMainMethod(strMethodName);
			if (pMethod) {
				pThreadInfo->m_ivResultRegister = pMethod->CallMainMethod(pValues, pThreadInfo);
			}
			else {
				pThreadInfo->m_ivResultRegister = GetIrisModule("Kernel")->CallClassMethod(strMethodName, pValues, pThreadInfo->m_pEnvrionmentRegister, pThreadInfo, CallerSide::Outside);
			}
		}
	}

	if (pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock) {
		IrisContextEnvironment* pTmpEnv = static_cast<IrisContextEnvironment*>(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetCurrentContextEnvironment());
		while (pTmpEnv) {
			--pTmpEnv->m_nReferenced;
			pTmpEnv = pTmpEnv->m_pUpperContextEnvironment;
		}
	}

	if (IrregularHappened(pThreadInfo)) {
		PopEnvironment(pThreadInfo);
	}

	return true;
}

bool IrisInterpreter::set_fld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	unsigned int nFields = vcVector[nCodePointer] & 0x00FF;
	IrisAM iaAM;

	for (unsigned i = 0; i < nFields; ++i) {
		iaAM = GetOneAM(vcVector, nCodePointer);
		pThreadInfo->m_lsFieldsRegister.push_back(m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex));
	}
	return true;
}

bool IrisInterpreter::clr_fld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_lsFieldsRegister.clear();
	return true;
}

bool IrisInterpreter::fld_load(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	IrisModule* pModule = nullptr;
	IrisAM iaAM;
	bool bResult = false;
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	iaAM = GetOneAM(vcVector, nCodePointer);
	auto nTargetIndex = iaAM.m_dwIndex;
	iaAM = GetOneAM(vcVector, nCodePointer);
	bool bHaveBegin = iaAM.m_dwIndex == 1;
	if (!pThreadInfo->m_lsFieldsRegister.empty()) {
		auto& ivResultRegister = pThreadInfo->m_ivResultRegister;
		auto pObject = static_cast<IrisObject*>(ivResultRegister.GetIrisObject());
		if (!pObject->GetClass()->GetInternClass()->IsModuleClass()) {
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::FieldCannotRoutedIrregular,
				pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
				"Invalid routing head, the routing head must be a Module.", pThreadInfo);
			return false;
		}
		auto pModule = IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>(ivResultRegister)->GetModule();
		
		bool bSearchResult = false;
		size_t nCount = 0;
		size_t nSize = pThreadInfo->m_lsFieldsRegister.size();
		IrisValue ivSearchResult;
		for (auto iField = pThreadInfo->m_lsFieldsRegister.begin(); iField != pThreadInfo->m_lsFieldsRegister.end(); ++iField) {
			++nCount;
			auto& ivResult = pModule->SearchConstance(*iField, bSearchResult);
			if (!bSearchResult) {
#if IR_USE_STL_STRING
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::FieldCannotRoutedIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Identifier of " + *iField + "not found.", pThreadInfo);
#else
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::FieldCannotRoutedIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Identifier of " + (*iField).GetSTLString() + " not found.", pThreadInfo);
#endif // IR_USE_STL_STRING
				return false;
			}
			auto pRouterObject = static_cast<IrisObject*>(ivResult.GetIrisObject());
			if (pRouterObject->GetClass()->GetInternClass()->IsModuleClass()) {
				pModule = IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>((IrisValue&)ivResult)->GetModule();
				ivSearchResult = ivResult;
			}
			else if (pRouterObject->GetClass()->GetInternClass()->IsClassClass() && nCount == nSize) {
				ivSearchResult = ivResult;
			}
			else {
				IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::FieldCannotRoutedIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Invalid routing.", pThreadInfo);
				return false;
			}
		}
		auto pSearchObject = static_cast<IrisObject*>(ivSearchResult.GetIrisObject());
		if (pSearchObject->GetClass()->GetInternClass()->IsClassClass()) {
			auto pClass = IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(ivSearchResult)->GetClass();
			pThreadInfo->m_ivResultRegister = pClass->SearchConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
		}
		else {
			auto pModule = IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>(ivSearchResult)->GetModule();
			pThreadInfo->m_ivResultRegister = pModule->SearchConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
		}
	}
	else if (pThreadInfo->m_lsFieldsRegister.empty() && bHaveBegin) {
		auto& ivResultRegister = pThreadInfo->m_ivResultRegister;
		auto pObject = static_cast<IrisObject*>(ivResultRegister.GetIrisObject());
		if (!pObject->GetClass()->GetInternClass()->IsModuleClass() && !pObject->GetClass()->GetInternClass()->IsClassClass()) {
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::FieldCannotRoutedIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Invalid routing head, the routing head must be a Module or a Class.", pThreadInfo);
			return false;
		}
		if (pObject->GetClass()->GetInternClass()->IsModuleClass()) {
			auto pModule = IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>(ivResultRegister)->GetModule();
			pThreadInfo->m_ivResultRegister = pModule->SearchConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
		}
		else {
			auto pClass = IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(ivResultRegister)->GetClass();
			pThreadInfo->m_ivResultRegister = pClass->SearchConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
		}
	}
	else {
		if (!pThreadInfo->m_pEnvrionmentRegister) {
			pThreadInfo->m_ivResultRegister = GetConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
		}
		else {
			switch (pThreadInfo->m_pEnvrionmentRegister->m_eType)
			{
			case IrisContextEnvironment::EnvironmentType::ClassDefineTime: 
			{
				IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
				if (pClass->GetUpperModule()) {
					pThreadInfo->m_ivResultRegister = pClass->GetUpperModule()->SearchConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
				}
				else {
					pThreadInfo->m_ivResultRegister = GetConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
				}
			}
				break;
			case IrisContextEnvironment::EnvironmentType::ModuleDefineTime:
			{
				IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
				if (pModule->GetUpperModule()) {
					pThreadInfo->m_ivResultRegister = pModule->GetUpperModule()->SearchConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
				}
				else {
					pThreadInfo->m_ivResultRegister = GetConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
				}
			}
				break;
			case IrisContextEnvironment::EnvironmentType::Runtime:
			{
				IrisObject* pObject = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject;
				if (pObject) {
					pThreadInfo->m_ivResultRegister = pObject->GetClass()->GetInternClass()->SearchConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
					if (!bResult) {
						IrisModule* pModule = pObject->GetClass()->GetInternClass()->GetUpperModule();
						pThreadInfo->m_ivResultRegister = pModule->SearchConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
					}
				}
				else {
					pThreadInfo->m_ivResultRegister = GetConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
				}
			}
				break;
			case IrisContextEnvironment::EnvironmentType::InterfaceDefineTime:
			{
				IrisInterface* pInterface = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pInterface;
				if (pInterface->GetUpperModule()) {
					pThreadInfo->m_ivResultRegister = pInterface->GetUpperModule()->SearchConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
				}
				else {
					pThreadInfo->m_ivResultRegister = GetConstance(m_pCurrentCompiler->GetIdentifier(nTargetIndex, nCurrentFileIndex), bResult);
				}
			}
			default:
				break;
			}
		}
	}

	if (!bResult) {
		// ** Error **
#if IR_USE_STL_STRING
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdentifierNotFoundIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Identifier of " + m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex) + " not found.", pThreadInfo);
#else
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdentifierNotFoundIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Identifier of " + m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex).GetSTLString() + " not found.", pThreadInfo);
#endif // IR_USE_STL_STRING
		return false;
	}

	return true;
}

bool IrisInterpreter::load_nil(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_ivResultRegister = m_ivNil;
	return true;
}

bool IrisInterpreter::load_true(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_ivResultRegister = m_ivTrue;
	return true;
}

bool IrisInterpreter::load_false(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_ivResultRegister = m_ivFalse;
	return true;
}

bool IrisInterpreter::load_self(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	if (!pThreadInfo->m_pEnvrionmentRegister
		|| pThreadInfo->m_pEnvrionmentRegister->m_eType != IrisContextEnvironment::EnvironmentType::Runtime
		|| !pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject
		|| pThreadInfo->m_pEnvrionmentRegister->m_bIsClosure) {
		// ** Error **
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::SelfPointerIrregular,
			pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
			"Statement of self must be used in INSTANCE METHOD OF CLASS.", pThreadInfo);
		return false;
	}

	IrisObject* pObject = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pCurObject;

	if (pObject->GetClass()->GetInternClass()->IsClassClass() || pObject->GetClass()->GetInternClass()->IsModuleClass()) {
		// ** Error **
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::SelfPointerIrregular,
			pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
			"Statement of self must be used in INSTANCE METHOD OF CLASS.", pThreadInfo);
		return false;
	}

	pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pObject);

	return true;
}

bool IrisInterpreter::imth_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
#if IR_USE_STL_STRING
	string strMethodName = "";
#else
	IrisInternString strMethodName = "";
#endif // IR_USE_STL_STRING
	IrisMethod::UserFunction* pUserFunction= nullptr;

	if (!BuildUserFunction((void**)&pUserFunction, vcVector, nCodePointer, strMethodName, pThreadInfo)) {
		return false;
	}

  	IrisMethod* pMethod = new IrisMethod(strMethodName, pUserFunction);

	if (!pThreadInfo->m_pEnvrionmentRegister) {
		AddMainMethod(strMethodName, pMethod);
	}
	else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ClassDefineTime) {
		IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
		pClass->AddInstanceMethod(pMethod);
	}
	else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ModuleDefineTime) {
		IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
		pModule->AddInstanceMethod(pMethod);
	}
	//else {
	//	// ** Error **
	//	return false;
	//}

	pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pMethod->GetMethodObject());

	return true;
}

bool IrisInterpreter::cmth_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
#if IR_USE_STL_STRING
	string strMethodName = "";
#else
	IrisInternString strMethodName = "";
#endif // IR_USE_STL_STRING
	IrisMethod::UserFunction* pUserFunction = nullptr;

	if (!BuildUserFunction((void**)&pUserFunction, vcVector, nCodePointer, strMethodName, pThreadInfo)) {
		return false;
	}

	IrisMethod* pMethod = new IrisMethod(strMethodName, pUserFunction);

	if (!pThreadInfo->m_pEnvrionmentRegister) {
		AddMainMethod(strMethodName, pMethod);
	}
	else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ClassDefineTime) {
		IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
		pClass->AddClassMethod(pMethod);
	}
	else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ModuleDefineTime) {
		IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
		pModule->AddClassMethod(pMethod);
	}
	//else {
	//	// ** Error **
	//	return false;
	//}

	pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pMethod->GetMethodObject());

	return true;
}

bool IrisInterpreter::blk_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	nCodePointer += 3;
	return true;
}

bool IrisInterpreter::end_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	nCodePointer += 3;
	return true;
}

bool IrisInterpreter::jfon(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	if (!(pThreadInfo->m_ivResultRegister == m_ivFalse || pThreadInfo->m_ivResultRegister == m_ivNil)) {
		nCodePointer += 3;
		return true;
	}

	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	nCodePointer += iaAM.m_dwIndex;

	return true;
}

bool IrisInterpreter::jmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	nCodePointer += iaAM.m_dwIndex;
	return true;
}

bool IrisInterpreter::ini_tm(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_ivTimerRegister = pThreadInfo->m_ivResultRegister;
	if (IrisDevUtil::GetNativePointer<IrisIntegerTag*>(pThreadInfo->m_ivTimerRegister)->m_nInteger <= 0) {
		pThreadInfo->m_bUnimitedLoopFlagRegister = true;
	}
	else {
		pThreadInfo->m_bUnimitedLoopFlagRegister = false;
	}
	return true;
}

bool IrisInterpreter::ini_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	//pThreadInfo->m_ivCounterRegister = GetIrisClass("Integer")->CreateInstanceByInstantValue(0);
	pThreadInfo->m_ivCounterRegister = IrisDevUtil::CreateInstanceByInstantValue(0);
	return true;
}

bool IrisInterpreter::cmp_tac(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	if (pThreadInfo->m_bUnimitedLoopFlagRegister) {
		pThreadInfo->m_ivResultRegister = m_ivTrue;
	}
	else {
		pThreadInfo->m_ivResultRegister = 
			IrisDevUtil::GetNativePointer<IrisIntegerTag*>(pThreadInfo->m_ivCounterRegister)->m_nInteger < IrisDevUtil::GetNativePointer<IrisIntegerTag*>(pThreadInfo->m_ivTimerRegister)->m_nInteger
			? m_ivTrue
			: m_ivFalse;
	}
	return true;
}

bool IrisInterpreter::inc_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	++IrisDevUtil::GetNativePointer<IrisIntegerTag*>(pThreadInfo->m_ivCounterRegister)->m_nInteger;
	return true;
}

bool IrisInterpreter::assign_log(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisCompiler* pCompiler = m_pCurrentCompiler;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	auto& strLog = pCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	bool bResult = false;
	if (pThreadInfo->m_pEnvrionmentRegister) {
		IrisValue& ivResult = (IrisValue&)pThreadInfo->m_pEnvrionmentRegister->GetVariableValue(strLog, bResult);
		ivResult.SetIrisObject(pThreadInfo->m_ivCounterRegister.GetIrisObject());
	}
	else {
		IrisValue& ivResult = (IrisValue&)GetOtherValue(strLog, bResult);
		ivResult.SetIrisObject(pThreadInfo->m_ivCounterRegister.GetIrisObject());
	}

	return true;
}

bool IrisInterpreter::brk(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
 {
	// Get Current Deep Index
	unsigned int nDeepIndex = GetTopDeepIndex(pThreadInfo);
	unsigned int nInstructor = 0;
	unsigned int nOperators = 0;
	//nCodePointer -= 2;
	//nCodePointer -= 1;
	IrisAM iaAM;
	while (true) {
		// end_def
		++nCodePointer;
		nInstructor = vcVector[++nCodePointer] >> 8;
		nOperators = vcVector[nCodePointer] & 0x00FF;
		if (nInstructor == 19) {
			iaAM = GetOneAM(vcVector, nCodePointer);
			if (nDeepIndex == iaAM.m_dwIndex) {
				break;
			}
		}
		else {
			nCodePointer += (nOperators * 3);
		}
	}
	return true;
}

bool IrisInterpreter::push_deep(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	PushDeepIndex(iaAM.m_dwIndex, pThreadInfo);
	return true;
}

bool IrisInterpreter::pop_deep(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	PopTopDeepIndex(pThreadInfo);
	return true;
}

bool IrisInterpreter::rtn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nEnder, IrisThreadInfo* pThreadInfo)
{
	nCodePointer -= 2;
	IrisAM iaAM;
	unsigned int nInstructor = 0;
	unsigned int nOperators = 0;

	const unsigned int& nCodeEnder = nEnder;

	//nCodePointer -= 2;
	unsigned int nIndex = GetTopDeepIndex(pThreadInfo);
	while (true) {
		if (++nCodePointer == nCodeEnder) {
			break;
		}
		++nCodePointer;
		nInstructor = vcVector[nCodePointer] >> 8;
		nOperators = vcVector[nCodePointer] & 0x00FF;
		// end_blk
		if (nInstructor == 19) {
			iaAM = GetOneAM(vcVector, nCodePointer);
			if (nIndex == iaAM.m_dwIndex) {
				PopTopDeepIndex(pThreadInfo);
				nIndex = GetTopDeepIndex(pThreadInfo);
			}
		}
		// pop_cnt
		else if (nInstructor == 45) {
			iaAM = GetOneAM(vcVector, nCodePointer);
			if (iaAM.m_dwIndex == nIndex) {
				PopCounter(pThreadInfo);
			}
		}
		// pop_tim
		else if (nInstructor == 46) {
			iaAM = GetOneAM(vcVector, nCodePointer);
			if (iaAM.m_dwIndex == nIndex) {
				PopTimer(pThreadInfo);
			}
		}
		// pop_unim
		else if (nInstructor == 48) {
			iaAM = GetOneAM(vcVector, nCodePointer);
			if (iaAM.m_dwIndex == nIndex) {
				PopUnlimitedLoopFlag(pThreadInfo);
			}
		}
		// pop_vsl
		else if (nInstructor == 50) {
			iaAM = GetOneAM(vcVector, nCodePointer);
			if (iaAM.m_dwIndex == nIndex) {
				PopVessle(pThreadInfo);
			}
		}
		// pop_iter
		else if (nInstructor == 52) {
			iaAM = GetOneAM(vcVector, nCodePointer);
			if (iaAM.m_dwIndex == nIndex) {
				PopIterator(pThreadInfo);
			}
		}
		else {
			nCodePointer += nOperators * 3;
		}
	}
	--nCodePointer;
	return true;
}

bool IrisInterpreter::ctn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{	// Get Current Deep Index
	unsigned int nDeepIndex = GetTopDeepIndex(pThreadInfo);
	unsigned int nInstructor = 0;
	unsigned int nOperators = 0;
	IrisAM iaAM;
	//nCodePointer -= 2;
	while (true) {
		// end_def
		++nCodePointer;
		nInstructor = vcVector[++nCodePointer] >> 8; 
		nOperators = vcVector[nCodePointer] & 0x00FF;
		if (nInstructor == 19) {
			iaAM = GetOneAM(vcVector, nCodePointer);
			if (nDeepIndex == iaAM.m_dwIndex) {
				break;
			}
		}
		else {
			nCodePointer += nOperators * 3;
		}
	}
	nCodePointer -= 10;
	return true;
}

bool IrisInterpreter::assign_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_ivVessleRegister = pThreadInfo->m_ivResultRegister;
	static_cast<IrisObject*>(pThreadInfo->m_ivVessleRegister.GetIrisObject())->Fix();
	return true;
}

bool IrisInterpreter::assign_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_ivIteratorRegister = pThreadInfo->m_ivResultRegister;
	return true;
}

bool IrisInterpreter::load_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_ivResultRegister = pThreadInfo->m_ivIteratorRegister;
	return true;
}

bool IrisInterpreter::jt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	if (pThreadInfo->m_ivResultRegister != m_ivTrue) {
		nCodePointer += 3;
		return true;
	}

	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	nCodePointer += iaAM.m_dwIndex;

	return true;
}

bool IrisInterpreter::assign_cmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	pThreadInfo->m_ivCompareRegister = pThreadInfo->m_ivResultRegister;
	return true;
}

bool IrisInterpreter::cmp_cmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	IrisValues ivsValues = { pThreadInfo->m_ivResultRegister };
	pThreadInfo->m_ivResultRegister = static_cast<IrisObject*>(pThreadInfo->m_ivCompareRegister.GetIrisObject())->CallInstanceFunction("==", &ivsValues, nullptr, pThreadInfo, CallerSide::Outside);
	return true;
}

bool IrisInterpreter::cre_cenv(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);

	auto pNewContextEnvironment = new IrisContextEnvironment();
	pNewContextEnvironment->m_pUpperContextEnvironment = pThreadInfo->m_pEnvrionmentRegister;

	switch (iaAM.m_dwIndex)
	{
	case 0:
		pNewContextEnvironment->m_eType = IrisContextEnvironment::EnvironmentType::ClassDefineTime;
		break;
	case 1:
		pNewContextEnvironment->m_eType = IrisContextEnvironment::EnvironmentType::ModuleDefineTime;
		break;
	case 2:
		pNewContextEnvironment->m_eType = IrisContextEnvironment::EnvironmentType::InterfaceDefineTime;
		break;
	default:
		break;
	}
	pThreadInfo->m_pEnvrionmentRegister = pNewContextEnvironment;
	return true;
}

bool IrisInterpreter::def_cls(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	
	IrisModule* pUpperModule = nullptr;
	auto& strClassName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	GetOneAM(vcVector, nCodePointer);

	auto pUpperEnvironment = pThreadInfo->m_pEnvrionmentRegister->m_pUpperContextEnvironment;

#if IR_USE_STL_STRING
	list<string> lsModules;
#else
	list<IrisInternString> lsModules;
#endif // IR_USE_STL_STRING
	if (pUpperEnvironment) {
		pUpperModule = pUpperEnvironment->m_uType.m_pModule;
		// ��ȡģ�������
		while (pUpperModule) {
			lsModules.push_front(pUpperModule->GetModuleName());
			pUpperModule = pUpperModule->GetUpperModule();
		}
	}
	lsModules.push_back(strClassName);

	IrisClass* pClass = GetIrisClass(lsModules);
	if (!pClass) {
		pClass = new IrisClass(strClassName, nullptr, pUpperEnvironment ? pUpperEnvironment->m_uType.m_pModule : nullptr, new IrisDummyClass());
		pClass->m_pExternClass->m_pInternClass = pClass;
		pClass->SetSuperClass(GetIrisClass("Object"));
		if (!RegistClass(lsModules, pClass->GetExternClass(), false)) {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::UnkownIrregular,
				pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
				"Oh, shit! An UNKNOWN ERROR has been lead to by YOU to Iris! What a SHIT unlucky man you are! Please don't approach Iris ANYMORE ! - The class CANNOT be registed to Iris.", pThreadInfo);
			return false;
		}
	}

	pClass->SetCompleted(false);
	pClass->ClearInvolvingModules();
	pClass->ClearJointingInterfaces();

	pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass = pClass;

	pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pClass->GetClassObject());

	return true;
}

bool IrisInterpreter::add_ext(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	// if not extends class
	if (!static_cast<IrisObject*>(pThreadInfo->m_ivResultRegister.GetIrisObject())->GetClass()->GetInternClass()->IsClassClass()) {
		// ** Error **
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::UnkownIrregular,
			pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
			"Oh, shit! An UNKNOWN ERROR has been lead to by YOU to Iris! What a SHIT unlucky man you are! Please don't approach Iris ANYMORE ! - An normal object has been extended to a Class.", pThreadInfo);
		return false;
	}

	IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
	pClass->SetSuperClass(IrisDevUtil::GetNativePointer<IrisClassBaseTag*>(pThreadInfo->m_ivResultRegister)->GetClass());

	return true;
}

bool IrisInterpreter::add_mld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	// if not involves module
	if (!static_cast<IrisObject*>(pThreadInfo->m_ivResultRegister.GetIrisObject())->GetClass()->GetInternClass()->IsModuleClass()) {
		// ** Error **
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::UnkownIrregular,
			pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
			"Oh, shit! An UNKNOWN ERROR has been lead to by YOU to Iris! What a SHIT unlucky man you are! Please don't approach Iris ANYMORE ! - A normal object has been involved with a Module.", pThreadInfo);
 		return false;
	}

	if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ClassDefineTime) {
		IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
		pClass->AddModule(IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>(pThreadInfo->m_ivResultRegister)->GetModule());
	}
	else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ModuleDefineTime) {
		IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
		pModule->AddModule(IrisDevUtil::GetNativePointer<IrisModuleBaseTag*>(pThreadInfo->m_ivResultRegister)->GetModule());
	}

	return true;
}

bool IrisInterpreter::add_inf(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{	// if not involves module
	if (!static_cast<IrisObject*>(pThreadInfo->m_ivResultRegister.GetIrisObject())->GetClass()->GetInternClass()->IsInterfaceClass()) {
		// ** Error **
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::UnkownIrregular,
			pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
			"Oh, shit! An UNKNOWN ERROR has been lead to by YOU to Iris! What a SHIT unlucky man you are! Please don't approach Iris ANYMORE ! - A normal object has been involved with a Interface.", pThreadInfo);
		return false;
	}

	if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ClassDefineTime) {
		IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
		pClass->AddInterface(IrisDevUtil::GetNativePointer<IrisInterfaceBaseTag*>(pThreadInfo->m_ivResultRegister)->GetInterface());
	}
	else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::ModuleDefineTime) {
		IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
		pModule->AddInvolvedInterface(IrisDevUtil::GetNativePointer<IrisInterfaceBaseTag*>(pThreadInfo->m_ivResultRegister)->GetInterface());
	}
	else if (pThreadInfo->m_pEnvrionmentRegister->m_eType == IrisContextEnvironment::EnvironmentType::InterfaceDefineTime) {
		auto pInterface = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pInterface;
		pInterface->AddInterface(IrisDevUtil::GetNativePointer<IrisInterfaceBaseTag*>(pThreadInfo->m_ivResultRegister)->GetInterface());
	}
	return true;
}

bool IrisInterpreter::push_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	PushCounter(pThreadInfo);
	return true;
}

bool IrisInterpreter::push_tim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	PushTimer(pThreadInfo);
	return true;
}

bool IrisInterpreter::pop_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	nCodePointer += 3;
	PopCounter(pThreadInfo);
	return true;
}

bool IrisInterpreter::pop_tim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	nCodePointer += 3;
	PopTimer(pThreadInfo);
	return true;
}

bool IrisInterpreter::push_unim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	PushUnlimitedLoopFlag(pThreadInfo);
	return true;
}

bool IrisInterpreter::pop_unim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	nCodePointer += 3;
	PopUnlimitedLoopFlag(pThreadInfo);
	return true;
}

bool IrisInterpreter::push_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	PushVessle(pThreadInfo);
	return true;
}

bool IrisInterpreter::pop_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	nCodePointer += 3;
	static_cast<IrisObject*>(pThreadInfo->m_ivVessleRegister.GetIrisObject())->Unfix();
	PopVessle(pThreadInfo);
	return true;
}

bool IrisInterpreter::push_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	PushIterator(pThreadInfo);
	return true;
}

bool IrisInterpreter::pop_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	nCodePointer += 3;
	PopIterator(pThreadInfo);
	return true;
}

bool IrisInterpreter::str_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	auto& strVariableName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
	string strSetterName;
#if IR_USE_STL_STRING
	strSetterName.assign(strVariableName.begin() + 1, strVariableName.end());
#else
	strSetterName.assign(strVariableName.GetSTLString().begin() + 1, strVariableName.GetSTLString().end());
#endif // IR_USE_STL_STRING
	strSetterName = "__set_" + strSetterName;

	iaAM = GetOneAM(vcVector, nCodePointer);
	auto& strParameterName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
	iaAM = GetOneAM(vcVector, nCodePointer);
	bool bWithBlock = iaAM.m_dwIndex == 0 ? false : true;
	iaAM = GetOneAM(vcVector, nCodePointer);

	IrisMethod* pMethod = nullptr;
	IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;

	if (bWithBlock) {
		IrisMethod::UserFunction* pUserFunction = new IrisMethod::UserFunction();
		pUserFunction->m_strVariableParameter = "";
		pUserFunction->m_lsParameters.push_back(strParameterName);
		pUserFunction->m_dwIndex = iaAM.m_dwIndex;

		// Block
		// �����һ��ָ�Ϊblk_def�򱨴�
		IR_BYTE bInstructor = vcVector[nCodePointer += 2] >> 8;
		if (bInstructor != 18) {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::UnkownIrregular,
				pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
				"Oh, shit! An UNKNOWN ERROR has been lead to by YOU to Iris! What a SHIT unlucky man you are! Please don't approach Iris ANYMORE ! Well, Iris surely DO NOT kown How STUPPID and UNBELIEVABLE an operation you have done this time.", pThreadInfo);
			return false;
		}
		iaAM = GetOneAM(vcVector, nCodePointer);
		unsigned int nIndex = iaAM.m_dwIndex;

		GetCodesFromBlock(nIndex, vcVector, nCodePointer, pUserFunction->m_icsBlockCodes);

		pMethod = new IrisMethod(strSetterName, pUserFunction);
		pUserFunction->m_icsBlockCodes.m_nBelongingFileIndex = nCurrentFileIndex;
	}
	else {
		IrisMethod::UserFunction* pUserFunction = new IrisMethod::UserFunction();
		pUserFunction->m_strVariableParameter = "";
		pUserFunction->m_lsParameters.push_back(strParameterName);
		pUserFunction->m_dwIndex = iaAM.m_dwIndex;
		pUserFunction->m_lsParameters.push_back("value");

		pMethod = new IrisMethod(strSetterName, pUserFunction, IrisMethod::MethodType::SetterMethod);

		nCodePointer += 4 + 1;
	}

	pClass->AddInstanceMethod(pMethod);
	pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pMethod->GetMethodObject());

	return true;
}

bool IrisInterpreter::gtr_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
#if IR_USE_STL_STRING
	const string& strVariableName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
#else
	auto& strVariableName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
#endif // IR_USE_STL_STRING
	string strGetterName;
#if IR_USE_STL_STRING
	strGetterName.assign(strVariableName.begin() + 1, strVariableName.end());
#else
	strGetterName.assign(strVariableName.GetSTLString().begin() + 1, strVariableName.GetSTLString().end());
#endif // IR_USE_STL_STRING
	strGetterName = "__get_" + strGetterName;

	iaAM = GetOneAM(vcVector, nCodePointer);
	bool bWithBlock = iaAM.m_dwIndex == 0 ? false : true;
	iaAM = GetOneAM(vcVector, nCodePointer);

	IrisMethod* pMethod = nullptr;
	IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;

	if (bWithBlock) {
		IrisMethod::UserFunction* pUserFunction = new IrisMethod::UserFunction();
		pUserFunction->m_strVariableParameter = "";
		pUserFunction->m_dwIndex = iaAM.m_dwIndex;

		// Block
		// �����һ��ָ�Ϊblk_def�򱨴�
		IR_BYTE bInstructor = vcVector[nCodePointer += 2] >> 8;
		if (bInstructor != 18) {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::UnkownIrregular,
				pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
				"Oh, shit! An UNKNOWN ERROR has been lead to by YOU to Iris! ���ΥЩ`�������ä������ˤʤ����裡�ʤ�ǥץ���ߥ󥰤��äƤ�Ρ����äȤ⤢�äƤʤ��Τˡ�һ������ԥ�`���`�ʤ�ƴ��餺�Ť���������o�j�^�����ʡ����Υ������g��", pThreadInfo);
			return false;
		}
		iaAM = GetOneAM(vcVector, nCodePointer);
		unsigned int nIndex = iaAM.m_dwIndex;

		GetCodesFromBlock(nIndex, vcVector, nCodePointer, pUserFunction->m_icsBlockCodes);
		pUserFunction->m_icsBlockCodes.m_nBelongingFileIndex = nCurrentFileIndex;

		pMethod = new IrisMethod(strGetterName, pUserFunction);
	}
	else {
		IrisMethod::UserFunction* pUserFunction = new IrisMethod::UserFunction();
		pUserFunction->m_strVariableParameter = "";
		pUserFunction->m_dwIndex = iaAM.m_dwIndex;

		pMethod = new IrisMethod(strGetterName, pUserFunction, IrisMethod::MethodType::GetterMethod);
		
		nCodePointer += 4 + 1;
	}

	pClass->AddInstanceMethod(pMethod);
	pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pMethod->GetMethodObject());
	
	return true;
}

bool IrisInterpreter::gstr_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	auto& strVariableName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
	string strSetterName;
#if IR_USE_STL_STRING
	strSetterName.assign(strVariableName.begin() + 1, strVariableName.end());
#else
	strSetterName.assign(strVariableName.GetSTLString().begin() + 1, strVariableName.GetSTLString().end());
#endif // IR_USE_STL_STRING
	strSetterName = "__set_" + strSetterName;

	string strGetterName;
#if IR_USE_STL_STRING
	strGetterName.assign(strVariableName.begin() + 1, strVariableName.end());
#else
	strGetterName.assign(strVariableName.GetSTLString().begin() + 1, strVariableName.GetSTLString().end());
#endif // IR_USE_STL_STRING
	strGetterName = "__get_" + strGetterName;

	IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;

	// Getter
	IrisMethod::UserFunction* pUserFunction = new IrisMethod::UserFunction();
	pUserFunction->m_strVariableParameter = "";
	pUserFunction->m_dwIndex = iaAM.m_dwIndex;
	IrisMethod* pMethod = new IrisMethod(strGetterName, pUserFunction, IrisMethod::MethodType::GetterMethod);
	pClass->AddInstanceMethod(pMethod);

	// Setter
	pUserFunction = new IrisMethod::UserFunction();
	pUserFunction->m_strVariableParameter = "";
	pUserFunction->m_dwIndex = iaAM.m_dwIndex;
	pUserFunction->m_lsParameters.push_back("value");
	pMethod = new IrisMethod(strSetterName, pUserFunction, IrisMethod::MethodType::SetterMethod);
	pClass->AddInstanceMethod(pMethod);

	return true;
}

bool IrisInterpreter::set_auth(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	auto& strMethodName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	iaAM = GetOneAM(vcVector, nCodePointer);
	IrisAuthorityEnvironment eEnv = (IrisAuthorityEnvironment)iaAM.m_dwIndex;
	iaAM = GetOneAM(vcVector, nCodePointer);
	IrisAuthorityTarget eTar = (IrisAuthorityTarget)iaAM.m_dwIndex;
	iaAM = GetOneAM(vcVector, nCodePointer);
	IrisAuthorityType eType = (IrisAuthorityType)iaAM.m_dwIndex;

	IrisMethod* pMethod = nullptr;
	if (eEnv == IrisAuthorityEnvironment::Class) {
		IrisClass* pClass = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pClass;
		if (eTar == IrisAuthorityTarget::ClassMethod) {
			//pMethod = pClass->GetCurrentClassMethod(IrisClass::SearchMethodType::ClassMethod, strMethodName);
			pMethod = static_cast<IrisObject*>(pClass->GetClassObject())->GetInstanceMethod(strMethodName);
		}
		else {
			//pMethod = pClass->GetCurrentClassMethod(IrisClass::SearchMethodType::InstanceMethod, strMethodName);
			pMethod = pClass->GetCurrentClassMethod(strMethodName);
		}
		if (!pMethod) {
			// **Error**
#if IR_USE_STL_STRING
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Method of " + strMethodName + " not found.", pThreadInfo);
#else
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Method of " + strMethodName.GetSTLString() + " not found.", pThreadInfo);
#endif // IR_USE_STL_STRING
			return false;
		}
		switch (eType)
		{
		case IrisAuthorityType::EveryOne:
			pMethod->SetMethodAuthority(IrisMethod::MethodAuthority::Everyone);
			break;
		case IrisAuthorityType::Relative:
			pMethod->SetMethodAuthority(IrisMethod::MethodAuthority::Relative);
			break;
		case IrisAuthorityType::Personal:
			pMethod->SetMethodAuthority(IrisMethod::MethodAuthority::Personal);
			break;
		}
	}
	else {
		IrisModule* pModule = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule;
		if (eTar == IrisAuthorityTarget::ClassMethod) {
			//pMethod = pModule->GetCurrentModuleMethod(IrisModule::SearchMethodType::ClassMethod, strMethodName);
			pMethod = static_cast<IrisObject*>(pModule->GetModuleObject())->GetInstanceMethod(strMethodName);
		}
		else {
			//pMethod = pModule->GetCurrentModuleMethod(IrisModule::SearchMethodType::InstanceMethod, strMethodName);
			pMethod = pModule->GetCurrentModuleMethod(strMethodName);
		}
		if (!pMethod) {
			// **Error**
			return false;
		}
		switch (eType)
		{
		case IrisAuthorityType::EveryOne:
			pMethod->SetMethodAuthority(IrisMethod::MethodAuthority::Everyone);
			break;
		case IrisAuthorityType::Relative:
			pMethod->SetMethodAuthority(IrisMethod::MethodAuthority::Relative);
			break;
		case IrisAuthorityType::Personal:
			pMethod->SetMethodAuthority(IrisMethod::MethodAuthority::Personal);
			break;
		}
	}

	pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pMethod->GetMethodObject());

	return true;
}

bool IrisInterpreter::def_mld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);

	IrisModule* pUpperModule = nullptr;
	auto& strModuleName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	GetOneAM(vcVector, nCodePointer);

	auto pUpperEnvironment = pThreadInfo->m_pEnvrionmentRegister->m_pUpperContextEnvironment;

#if IR_USE_STL_STRING
	list<string> lsModules;
#else
	list<IrisInternString> lsModules;
#endif // IR_USE_STL_STRING
	if (pUpperEnvironment) {
		pUpperModule = pUpperEnvironment->m_uType.m_pModule;
		// ��ȡģ�������
		while (pUpperModule) {
			lsModules.push_front(pUpperModule->GetModuleName());
			pUpperModule = pUpperModule->GetUpperModule();
		}
	}
	lsModules.push_back(strModuleName);

	IrisModule* pModule = GetIrisModule(lsModules);
	if (!pModule) {
		pModule = new IrisModule(strModuleName, pUpperEnvironment ? pUpperEnvironment->m_uType.m_pModule : nullptr);
		pModule->m_pExternModule = new IrisDummyModule();
		pModule->m_pExternModule->m_pInternModule = pModule;
		if (!RegistModule(lsModules, pModule->GetExternModule(), false)) {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::UnkownIrregular,
				pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
				"Oh, shit! An UNKNOWN ERROR has been lead to by YOU to Iris! What a SHIT unlucky man you are! Please don't approach Iris ANYMORE ! - The interface CANNOT be registed to Iris.", pThreadInfo);
			return false;
		}
	}

	pModule->ClearInvolvingModules();
	//pModule->ClearJointedInterfaces();
	pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pModule = pModule;
	
	pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pModule->GetModuleObject());

	return true;
}

bool IrisInterpreter::def_inf(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);

	IrisModule* pUpperModule = nullptr;
	auto& strInterfaceName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	GetOneAM(vcVector, nCodePointer);

	auto pUpperEnvironment = pThreadInfo->m_pEnvrionmentRegister->m_pUpperContextEnvironment;

#if IR_USE_STL_STRING
	list<string> lsModules;
#else
	list<IrisInternString> lsModules;
#endif // IR_USE_STL_STRING
	if (pUpperEnvironment) {
		pUpperModule = pUpperEnvironment->m_uType.m_pModule;
		// ��ȡģ�������
		while (pUpperModule) {
			lsModules.push_front(pUpperModule->GetModuleName());
			pUpperModule = pUpperModule->GetUpperModule();
		}
	}
	lsModules.push_back(strInterfaceName);

	IrisInterface* pInterface = GetIrisInterface(lsModules);
	if (!pInterface) {
		pInterface = new IrisInterface(strInterfaceName, pUpperEnvironment ? pUpperEnvironment->m_uType.m_pModule : nullptr);
		//pModule->m_pExternModule = new IrisDummyModule();
		pInterface->m_pExternInterface = new IrisDummyInterface();
		pInterface->m_pExternInterface->m_pInternInterface = pInterface;

		if (!RegistInterface(lsModules, pInterface->GetExternInterface(), false)) {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::UnkownIrregular,
				pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex,
				"Oh, shit! An UNKNOWN ERROR has been lead to by YOU to Iris! What a SHIT unlucky man you are! Please don't approach Iris ANYMORE ! - The interface CANNOT be registed to Iris.", pThreadInfo);
			return false;
		}
	}

	pInterface->ClearJointingInterfaces();

	pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pInterface = pInterface;

	pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pInterface->GetInterfaceObject());

	return true;
}

bool IrisInterpreter::def_infs(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisInterface* pInterface = pThreadInfo->m_pEnvrionmentRegister->m_uType.m_pInterface;
	unsigned int nParameterCount = (vcVector[nCodePointer] & 0x00FF) - 2;

	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	auto& strMethodName = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);

	//iaAM = GetOneAM(vcVector, nCodePointer);
	//int nParamSize = nParameterCount;
	
	nCodePointer += nParameterCount * 3;

	iaAM = GetOneAM(vcVector, nCodePointer);
	bool bHaveVariableParams = iaAM.m_dwIndex == 1 ? true : false;

	pInterface->AddInterfaceFunctionDeclare(strMethodName, nParameterCount, bHaveVariableParams);

	return true;
}

bool IrisInterpreter::cblk_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	unsigned int nPrametersCount = (vcVector[nCodePointer] & 0x00FF) - 1 - 1;
#if IR_USE_STL_STRING
	list<string> lsParameters;
#else
	list<IrisInternString> lsParameters;
#endif // IR_USE_STL_STRING
	vector<IR_WORD> vcCodes;
	IrisCodeSegment icsCodeSegment;

	IrisAM iaAM;
	for (unsigned int i = 0; i < nPrametersCount; ++i) {
		iaAM = GetOneAM(vcVector, nCodePointer);
		lsParameters.push_back(m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex));
	}

	// variable parameter
	auto iaAMVP = GetOneAM(vcVector, nCodePointer);

	iaAM = GetOneAM(vcVector, nCodePointer);
	unsigned int nIndex = iaAM.m_dwIndex;

	GetCodesFromBlock(nIndex, vcVector, nCodePointer, icsCodeSegment);

	IrisClosureBlock* pBlock = new IrisClosureBlock(
		pThreadInfo->m_pEnvrionmentRegister,
		lsParameters, 
		iaAMVP.m_dwIndex == -1 ? "" : m_pCurrentCompiler->GetIdentifier(iaAMVP.m_dwIndex, nCurrentFileIndex),
		icsCodeSegment.m_nStartPointer, 
		icsCodeSegment.m_nEndPointer,
		*icsCodeSegment.m_pWholeCodes,
		nCurrentFileIndex,
		nIndex,
		pThreadInfo
		);
	pThreadInfo->m_pClosureBlockRegister = pBlock;

	return true;
}

bool IrisInterpreter::blk(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo) {
	if (pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock) {
		return RunCode(*pThreadInfo->m_pEnvrionmentRegister->m_pWithBlock->m_pWholeCodes, pThreadInfo->m_pEnvrionmentRegister->m_pWithBlock->m_nStartPointer, pThreadInfo->m_pEnvrionmentRegister->m_pWithBlock->m_nEndPointer, pThreadInfo);
	}
	else {
		return RunCode(*pThreadInfo->m_pEnvrionmentRegister->m_pWithoutBlock->m_pWholeCodes, pThreadInfo->m_pEnvrionmentRegister->m_pWithoutBlock->m_nStartPointer, pThreadInfo->m_pEnvrionmentRegister->m_pWithoutBlock->m_nEndPointer, pThreadInfo);
	}
}

bool IrisInterpreter::reg_irp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	IrisContextEnvironment::IrregularProcessProgram ipProgram;// = new IrisContextEnvironment::IrregularProcessProgram();

	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	bool bWithIgnoreBlock = iaAM.m_dwIndex == 1;

	iaAM = GetOneAM(vcVector, nCodePointer);

	++nCodePointer;
	++nCodePointer;
	iaAM = GetOneAM(vcVector, nCodePointer);
	unsigned int nIndex = iaAM.m_dwIndex;
	GetCodesFromBlock(nIndex, vcVector, nCodePointer, ipProgram.m_icsOrderCodes);

	++nCodePointer;
	++nCodePointer;
	iaAM = GetOneAM(vcVector, nCodePointer);
	nIndex = iaAM.m_dwIndex;
	GetCodesFromBlock(nIndex, vcVector, nCodePointer, ipProgram.m_icsServeCodes);

	if (bWithIgnoreBlock) {
		++nCodePointer;
		++nCodePointer;
		iaAM = GetOneAM(vcVector, nCodePointer);
		nIndex = iaAM.m_dwIndex;
		GetCodesFromBlock(nIndex, vcVector, nCodePointer, ipProgram.m_icsIgnoreCodes);
	}
	
	RunCode(*ipProgram.m_icsOrderCodes.m_pWholeCodes, ipProgram.m_icsOrderCodes.m_nStartPointer, ipProgram.m_icsOrderCodes.m_nEndPointer, pThreadInfo);

	if (IrregularHappened(pThreadInfo)) {
		UnregistIrregular(pThreadInfo);
		RunCode(*ipProgram.m_icsServeCodes.m_pWholeCodes, ipProgram.m_icsServeCodes.m_nStartPointer, ipProgram.m_icsServeCodes.m_nEndPointer, pThreadInfo);
	}

	if (bWithIgnoreBlock) {
		bool bTmp = pThreadInfo->m_bIrregularHappenedRegister;
		pThreadInfo->m_bIrregularHappenedRegister = false;
		if (!RunCode(*ipProgram.m_icsIgnoreCodes.m_pWholeCodes, ipProgram.m_icsIgnoreCodes.m_nStartPointer, ipProgram.m_icsIgnoreCodes.m_nEndPointer, pThreadInfo)) {
			return false;
		}

		pThreadInfo->m_bIrregularHappenedRegister = bTmp;
	}

	return true;
}

bool IrisInterpreter::ureg_irp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	nCodePointer += 3;
	return true;
}

bool IrisInterpreter::assign_ir(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto nCurrentFileIndex = pThreadInfo->m_nCurrentFileIndex;
	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);

	auto& strLocalValue = m_pCurrentCompiler->GetIdentifier(iaAM.m_dwIndex, nCurrentFileIndex);
	bool bResult = false;
	if (pThreadInfo->m_pEnvrionmentRegister) {
		IrisValue& ivResult = (IrisValue&)pThreadInfo->m_pEnvrionmentRegister->GetVariableValue(strLocalValue, bResult);
		ivResult.SetIrisObject(pThreadInfo->m_ivIrregularObjectRegister.GetIrisObject());
	}
	else {
		IrisValue& ivResult = (IrisValue&)GetOtherValue(strLocalValue, bResult);
		ivResult.SetIrisObject(pThreadInfo->m_ivIrregularObjectRegister.GetIrisObject());
	}

	pThreadInfo->m_ivResultRegister = pThreadInfo->m_ivIrregularObjectRegister;

	return true;
}

bool IrisInterpreter::grn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	RegistIrregular(pThreadInfo->m_ivResultRegister, pThreadInfo);
	return true;
}

bool IrisInterpreter::spr(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo)
{
	if (pThreadInfo->m_pEnvrionmentRegister->m_pUpperContextEnvironment->m_eType != IrisContextEnvironment::EnvironmentType::Runtime) {
		return false;
	}

	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
	size_t nParameters = iaAM.m_dwIndex;

	auto pUpperEnvironment = pThreadInfo->m_skEnvironmentStack.back();

	IrisMethod* pMethod = pUpperEnvironment->m_pCurMethod;
	IrisObject* pObject = pUpperEnvironment->m_uType.m_pCurObject;
	//���Ҹ����ͬ������
	auto& strMethodName = pMethod->GetMethodName();

	if (!pObject
		|| pObject->GetClass()->GetInternClass()->IsClassClass()
		|| pObject->GetClass()->GetInternClass()->IsModuleClass()
		|| pObject->GetClass()->GetInternClass()->IsInterfaceClass()) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodCanSuperIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Super can only used in an instance method whose super class has the one with the same name.", pThreadInfo);
		return false;
	}

	IrisValues ivsValues(nParameters);
	size_t i = 0;
	for (auto it = pThreadInfo->m_stStack.m_lsStack.rbegin(); i < nParameters; ++i, ++it) {
		ivsValues[nParameters - i - 1] = *it;
	}
	
	IrisMethod* pSuperMethod = nullptr;

	if (pObject->GetClass()->GetInternClass()->IsClassClass()) {
		IrisObject* pClassObject = pObject;
		IrisClass* pClass = nullptr;

		do {
			pClass = static_cast<IrisClassBaseTag*>(pClassObject->GetNativeObject())->GetClass()->GetSuperClass();
			pClassObject = static_cast<IrisObject*>(pClass->GetClassObject());
			pSuperMethod = pClassObject->GetInstanceMethod(strMethodName);

			// ��鸸�������ģ��
			if (!pSuperMethod) {
				auto& hsModules = pClass->GetModules();
				for (auto& module : hsModules) {
					auto pModule = module;
					if (pSuperMethod = pModule->GetModuleInstanceMethod(strMethodName)) {
						break;
					}
				}
			}
		} while (!pClass->IsClassClass() && !pSuperMethod);
	}

	if (!pSuperMethod) {
		bool bResult = false;
		IrisClass* pSuperClass = pObject->GetClass()->GetInternClass()->GetSuperClass();

		if (!pSuperClass) {
			// ** Error **
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodCanSuperIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Super can only used in a instance method whose super class has the one with the same name.", pThreadInfo);
			return false;
		}

		pSuperMethod = pSuperClass->GetMethod(strMethodName, bResult);
	}

	if (pSuperMethod) {
		if (pThreadInfo->m_pClosureBlockRegister) {
			pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock = pThreadInfo->m_pClosureBlockRegister;
			pThreadInfo->m_pClosureBlockRegister = nullptr;

			auto ivObj = IrisDevUtil::CreateInstanceByInstantValue(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock);
			pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->SetNativeObject(static_cast<IrisObject*>(ivObj.GetIrisObject()));

			IrisContextEnvironment* pTmpEnv = static_cast<IrisContextEnvironment*>(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetCurrentContextEnvironment());
			while (pTmpEnv) {
				++pTmpEnv->m_nReferenced;
				pTmpEnv = pTmpEnv->m_pUpperContextEnvironment;
			}
		}

		IrisValue ivObj;
		ivObj.SetIrisObject(pObject);
		if (nParameters > 0) {
			pThreadInfo->m_ivResultRegister = pSuperMethod->Call(ivObj, &ivsValues, pThreadInfo->m_pEnvrionmentRegister, pThreadInfo);
		}
		else {
			pThreadInfo->m_ivResultRegister = pSuperMethod->Call(ivObj, nullptr, pThreadInfo->m_pEnvrionmentRegister, pThreadInfo);
		}

		if (pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock) {
			IrisContextEnvironment* pTmpEnv = static_cast<IrisContextEnvironment*>(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetCurrentContextEnvironment());
			while (pTmpEnv) {
				--pTmpEnv->m_nReferenced;
				pTmpEnv = pTmpEnv->m_pUpperContextEnvironment;
			}
		}

		if (IrregularHappened(pThreadInfo)) {
			PopEnvironment(pThreadInfo);
		}
	}
	else {
		// ** Error **
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::NoMethodCanSuperIrregular, pThreadInfo->m_nCurrentLineNumber, pThreadInfo->m_nCurrentFileIndex, "Super can only used in an instance method whose super class has the one with the same name.", pThreadInfo);
		return false;
	}

	return true;
}

bool IrisInterpreter::load_cast(vector<IR_WORD>& vcVector, unsigned int & nCodePointer, IrisThreadInfo* pThreadInfo)
{
	auto pClosureBlock = pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock;

	if (pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock) {
		pThreadInfo->m_ivResultRegister = IrisValue::WrapObjectPointerToIrisValue(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetNativeObject());
	}
	else {
		pThreadInfo->m_ivResultRegister = IrisDevUtil::Nil();
	}

	return true;
}

//bool IrisInterpreter::cast(vector<IR_WORD>& vcVector, unsigned int& nCodePointer) {
//	auto pThreadInfo = IrisDevUtil::GetCurrentThreadInfo();
//	IrisAM iaAM = GetOneAM(vcVector, nCodePointer);
//	const int nParameters = iaAM.m_dwIndex;
//
//	IrisValues* pValues = nullptr;
//	IrisValues ivsValues(nParameters);
//	if (nParameters) {
//		int i = 0;
//		//for (auto it = m_stStack.m_lsStack.rbegin(); i < nParameters; ++i, ++it) {
//		//	ivsValues[nParameters - i - 1] = *it;
//		//}
//		ivsValues.GetVector().assign(pThreadInfo->m_stStack.m_lsStack.begin() + pThreadInfo->m_stStack.m_lsStack.size() - nParameters, pThreadInfo->m_stStack.m_lsStack.end());
//		pValues = &ivsValues;
//	}
//
//	pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->Excute(pValues);
//	
//	return true;
//}