#include "IrisInterpreter/IrisNativeClasses/IrisObjectBase.h"


IrisValue IrisObjectBase::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return ivObj;
}

IrisValue IrisObjectBase::GetClass(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IIrisObject* pObject = IrisDevUtil::GetNativeObjectPointer(ivObj);
	return IrisValue::WrapObjectPointerToIrisValue(pObject);
}

IrisValue IrisObjectBase::GetObjectID(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	return IrisDevUtil::CreateInt(IrisDevUtil::GetObjectID(ivObj));
}

IrisValue IrisObjectBase::ToString(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	//const string& strClassName = static_cast<IrisObject*>(ivObj.GetIrisObject())->GetClass()->GetInternClass()->GetClassName();
	const string& strClassName = IrisDevUtil::GetNameOfClass(IrisDevUtil::GetClassOfObject(ivObj));
	IrisValue ivStringObjectID = IrisDevUtil::CallMethod(ivObj, "__get_object_id", nullptr, pContextEnvironment, pThreadInfo);
	const string& strObjectID = IrisDevUtil::GetNativePointer<IrisIntegerTag*>(ivStringObjectID)->ToString();
	string strOutString = "<" + strClassName + ":" + strObjectID + ">";
	return IrisDevUtil::CreateString(strOutString.c_str());
}

IrisValue IrisObjectBase::Equal(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	const IrisValue& ivDestObj = static_cast<IrisValues*>(ivsValues)->GetValue(0);
	if (ivObj == ivDestObj) {
		return IrisDevUtil::True();
	}
	else {
		return IrisDevUtil::False();
	}
}

IrisValue IrisObjectBase::NotEqual(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisValue ivResult = IrisDevUtil::CallMethod(ivObj, "==", ivsValues, pContextEnvironment, pThreadInfo);
	if (ivResult == IrisDevUtil::True()) {
		return IrisDevUtil::False();
	}
	else {
		return IrisDevUtil::True();
	}
}

IrisValue IrisObjectBase::LogicOr(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisValue& ivDest = (IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0);
	// 左右两边只要有一个不为nil或false那么就为true
	if (ivObj != IrisDevUtil::False() && ivObj != IrisDevUtil::Nil()) {
		return IrisDevUtil::True();
	}
	else if (ivDest != IrisDevUtil::False() && ivDest != IrisDevUtil::Nil()) {
		return IrisDevUtil::True();
	}
	else {
		return IrisDevUtil::False();
	}
}

IrisValue IrisObjectBase::LogicAnd(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	IrisValue& ivDest = (IrisValue&)static_cast<IrisValues*>(ivsValues)->GetValue(0);
	// 左右两边只要有一个为nil或false那么就为false
	if (ivObj == IrisDevUtil::False() || ivObj == IrisDevUtil::Nil()) {
		return IrisDevUtil::False();
	}
	//else if (ivDest == IrisDevUtil::False() && ivDest == IrisDevUtil::Nil()) {
	else if (ivDest == IrisDevUtil::False() || ivDest == IrisDevUtil::Nil()) {
		return IrisDevUtil::False();
	}
	else {
		return IrisDevUtil::True();
	}
}

IrisValue IrisObjectBase::Fix(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto pObject = static_cast<IrisObject*>(ivObj.GetIrisObject());
	pObject->Fix();
	return IrisDevUtil::Nil();
}

IrisValue IrisObjectBase::Unfix(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto pObject = static_cast<IrisObject*>(ivObj.GetIrisObject());
	pObject->Unfix();
	return IrisDevUtil::Nil();
}

IrisValue IrisObjectBase::IsFixed(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment, IIrisThreadInfo* pThreadInfo) {
	auto pObject = static_cast<IrisObject*>(ivObj.GetIrisObject());
	return pObject->IsFixed() ? IrisDevUtil::True() : IrisDevUtil::False();
}

IrisObjectBase::IrisObjectBase()
{
}


IrisObjectBase::~IrisObjectBase()
{
}
